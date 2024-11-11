use crate::ffitest::lean_helpers::{self, rust_string_to_lean};
use lean_sys::*;
use once_cell::sync::OnceCell;
use std::{collections::HashMap, error::Error, fmt::Display, sync::Mutex};

// maps from Address (String) -> Message (String)
static GLOBAL_MESSAGE_HASHTBL: OnceCell<Mutex<HashMap<String, String>>> = OnceCell::new();

#[no_mangle]
pub unsafe extern "C" fn get_node_value(node_address: *mut lean_object) -> *mut lean_object {
    let ht = GLOBAL_MESSAGE_HASHTBL
        .get()
        .expect("global message hashtbl should be initialized")
        .lock()
        .unwrap();
    let node_address_rust = lean_helpers::lean_string_to_rust(node_address);
    let message_rust = ht
        .get(&node_address_rust)
        .expect("node should always have a message")
        .clone();
    rust_string_to_lean(message_rust)
}

pub mod lean {

    use crate::ffitest::lean_helpers::{self, lean_string_to_rust, rust_string_to_lean};
    use lean_sys::*;
    use std::{collections::HashMap, sync::Mutex};

    use super::GLOBAL_MESSAGE_HASHTBL;

    #[link(name = "Protocol")]
    extern "C" {
        // https://doc.rust-lang.org/reference/items/external-blocks.html#the-link_name-attribute
        #[link_name = "initialize_Protocol"]
        pub fn initialize(builtin: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;

        fn create_protocol(node_arr: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        fn create_packet(
            src: lean_sys::lean_obj_arg,
            dst: lean_sys::lean_obj_arg,
            msg: lean_sys::lean_obj_arg,
            consumed: u8,
        ) -> lean_sys::lean_obj_res;
        fn create_message(
            tag: usize,
            originator: lean_sys::lean_obj_arg,
            r: usize,
            v: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
        fn init_node_state(
            p: lean_sys::lean_obj_arg,
            node_address: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
        fn send_message(
            p: lean_sys::lean_obj_arg,
            node_state: lean_sys::lean_obj_arg,
            round: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
        fn handle_message(
            p: lean_sys::lean_obj_arg,
            node_state: lean_sys::lean_obj_arg,
            src: lean_sys::lean_obj_arg,
            msg: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
    }

    // note: even though the lean representation looks identical to this enum type,
    // the memory representation is different.
    // since `USize` fields are ordered AFTER `lean_object` fields, each constructor would look like:
    // | EchoMsg { originator: String, v: String, r: usize }
    // and we'll have to deconstruct it in that order.
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub enum Message {
        InitialMsg {
            r: usize,
            v: String,
        },
        EchoMsg {
            originator: String,
            r: usize,
            v: String,
        },
        VoteMsg {
            originator: String,
            r: usize,
            v: String,
        },
    }
    impl std::fmt::Display for Message {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Message::InitialMsg { r, v } => write!(f, "InitialMsg @ round {}: {}", r, v),
                Message::EchoMsg { originator, r, v } => {
                    write!(f, "EchoMsg from {} @ round {}: {}", originator, r, v)
                }
                Message::VoteMsg { originator, r, v } => {
                    write!(f, "VoteMsg from {} @ round {}: {}", originator, r, v)
                }
            }
        }
    }

    impl Message {
        pub unsafe fn from_lean(msg_lean: *mut lean_object) -> Self {
            let tag = lean_ptr_tag(msg_lean);
            let mut current_field_id = 0;

            // only EchoMsg and VoteMsg have the originator fields.
            let mut originator: String = String::new();
            if tag == 1 || tag == 2 {
                originator = lean_string_to_rust(lean_ctor_get(msg_lean, current_field_id));
                current_field_id += 1;
            }

            let v = lean_string_to_rust(lean_ctor_get(msg_lean, current_field_id));
            current_field_id += 1;
            let r = lean_ctor_get_usize(msg_lean, current_field_id);

            // free the lean packet
            lean_dec(msg_lean);

            // construct Rust message
            match tag {
                0 => Message::InitialMsg { r, v },
                1 => Message::EchoMsg { originator, r, v },
                2 => Message::VoteMsg { originator, r, v },
                _ => panic!("unexpected tag"),
            }
        }

        // Takes ownership of the Rust Message.
        pub unsafe fn to_lean(self) -> *mut lean_object {
            let tag: usize;
            let originator_r: String;
            let r_r: usize;
            let v_r: String;

            match self {
                Self::InitialMsg { r, v } => {
                    tag = 0;
                    // hacky way to include this.
                    // will not be used on the Lean side.
                    originator_r = String::new();
                    r_r = r;
                    v_r = v;
                }
                Self::EchoMsg { originator, r, v } => {
                    tag = 1;
                    originator_r = originator;
                    r_r = r;
                    v_r = v;
                }
                Self::VoteMsg { originator, r, v } => {
                    tag = 2;
                    originator_r = originator;
                    r_r = r;
                    v_r = v;
                }
            };

            create_message(
                tag,
                rust_string_to_lean(originator_r),
                r_r,
                rust_string_to_lean(v_r),
            )
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct Packet {
        src: String,
        pub dst: String,
        pub msg: Message,
        consumed: bool,
    }

    impl std::fmt::Display for Packet {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "packet from {} to {} with message '{}' (consumed: {})",
                self.src, self.dst, self.msg, self.consumed
            )
        }
    }

    impl Packet {
        // TODO: check if the convention should be `from_lean` or `of_lean`.
        pub unsafe fn from_lean(packet_lean: *mut lean_object) -> Self {
            let src_lean = lean_ctor_get(packet_lean, 0);
            let dst_lean = lean_ctor_get(packet_lean, 1);
            let msg_lean = lean_ctor_get(packet_lean, 2);
            let consumed_lean_offset: std::ffi::c_uint =
                (3 * lean_helpers::VOID_PTR_SIZE).try_into().unwrap();
            let consumed_lean = lean_ctor_get_uint8(packet_lean, consumed_lean_offset);

            let src = lean_string_to_rust(src_lean);
            let dst = lean_string_to_rust(dst_lean);
            let msg = Message::from_lean(msg_lean);
            // no formal way to cast u8 to bool, so we do this instead
            let consumed: bool = consumed_lean != 0;

            // free the lean packet
            lean_dec(packet_lean);

            Packet {
                src,
                dst,
                msg,
                consumed,
            }
        }

        // Takes ownership of the rust packet.
        pub unsafe fn to_lean(self) -> *mut lean_object {
            create_packet(
                rust_string_to_lean(self.src),
                rust_string_to_lean(self.dst),
                Message::to_lean(self.msg),
                self.consumed as u8,
            )
        }
    }

    pub struct Protocol {
        protocol: *mut lean_object,
        node_state: *mut lean_object,
        round: *mut lean_object,
    }

    impl Protocol {
        pub unsafe fn create(node_list: Vec<String>, address: String) -> Self {
            // initialize protocol
            let node_list_raw: Vec<usize> = node_list
                .into_iter()
                .map(|s| {
                    let c_str_s = std::ffi::CString::new(s).unwrap();
                    c_str_s.as_ptr() as usize
                })
                .collect();
            let node_array_lean = lean_helpers::rust_usize_vec_to_lean_array(node_list_raw);
            let protocol = create_protocol(node_array_lean);

            // initialize this node's state
            let node_address_lean = rust_string_to_lean(address);
            let node_state = init_node_state(protocol, node_address_lean);

            // initialize the global message hashtbl
            GLOBAL_MESSAGE_HASHTBL
                .set(Mutex::new(HashMap::new()))
                .expect("should be able to init global hashtbl");

            // initialize round to 0
            let round = lean_usize_to_nat(0);

            Protocol {
                protocol,
                node_state,
                round,
            }
        }

        pub unsafe fn send_message(&mut self, address: String, message: String) -> Vec<Packet> {
            // update the message db with the current message
            let mut ht = GLOBAL_MESSAGE_HASHTBL
                .get()
                .expect("expected global message db to be initialized")
                .lock()
                .unwrap();
            ht.insert(address, message);

            // send the InitialMessage
            let state_and_packets = send_message(self.protocol, self.node_state, self.round);

            // deconstruct new protocol state
            assert!(lean_is_ctor(state_and_packets));
            assert!(lean_ctor_num_objs(state_and_packets) == 2);
            let new_state = lean_ctor_get(state_and_packets, 0);

            // update local state
            lean_dec(self.node_state);
            self.node_state = new_state;

            // deconstruct lean packets into rust
            let packets_arr_lean = lean_ctor_get(state_and_packets, 1);
            assert!(lean_is_array(packets_arr_lean));

            let n_packets: usize = lean_array_size(packets_arr_lean); // borrowing arr here

            let mut packets_to_send = Vec::new();
            for i in 0..n_packets {
                let packet_lean = lean_array_uget(packets_arr_lean, i); // borrow the lean packet

                // unmarshall the packet into rust and send it over the network.
                let packet_rust = Packet::from_lean(packet_lean);
                packets_to_send.push(packet_rust);

                lean_dec(packet_lean); // return the lean packet.
            }

            // increment round
            // note: this is maintained per-node for now, but eventually we may want some way
            // to broadcast the fact that we're starting a new round to all nodes?
            self.round = lean_nat_succ(self.round);

            packets_to_send
        }

        // pub unsafe fn handle_packet
    }
}

// for RB, we send all packets via `Request`s, and acknowledge receiving a packet
// via a `Response`.`
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RBRequest {
    pub packet: lean::Packet,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum RBResponse {
    Ack,
}

// may want to consider using a crate like `derive_more` to help us derive
// `Display` here.
impl Display for RBRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request: {}", self.packet)
    }
}

impl Display for RBResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "response: ack")
    }
}

impl Error for RBResponse {}
