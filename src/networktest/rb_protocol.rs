use crate::ffitest::lean_helpers::{self, rust_string_to_lean, Mode};
use lean_sys::*;
use once_cell::sync::OnceCell;
use std::{collections::HashMap, error::Error, fmt::Display, sync::Mutex};

// TODO: an alternative implementation for the global message hashtable would be to store global state
// either in the IO Monad or some form of State monad (StateM).
// the benefit of this is that state lives exactly in one place (NodeState) as opposed to being split across
// the NodeState and the global state on the Rust side, at the cost of additional hassle of working with monads.
// i should investigate this in the future.
//
// maps from Address (String) -> Message (String)
static GLOBAL_MESSAGE_HASHTBL: OnceCell<Mutex<HashMap<String, String>>> = OnceCell::new();

#[no_mangle]
pub unsafe extern "C" fn get_node_value(node_address: *mut lean_object) -> *mut lean_object {
    // println!("[rb_protocol::get_node_value] (extern) called");
    let ht = GLOBAL_MESSAGE_HASHTBL
        .get()
        .expect("global message hashtbl should be initialized")
        .lock()
        .unwrap();
    // println!("[rb_protocol::get_node_value] (extern) global hashtbl acquired");
    let node_address_rust = lean_helpers::lean_string_to_rust(node_address, Mode::Owned);
    let message_rust = ht
        .get(&node_address_rust)
        .expect("node should always have a message")
        .clone();
    // println!(
    //     "[rb_protocol::get_node_value] (extern) returning {}",
    //     &message_rust
    // );
    rust_string_to_lean(message_rust)
}

pub mod lean {

    use crate::ffitest::lean_helpers::{self, lean_string_to_rust, rust_string_to_lean, Mode};
    use lean_sys::*;
    use std::{collections::HashMap, sync::Mutex};

    use super::GLOBAL_MESSAGE_HASHTBL;

    // note: we link with `ProtocolFat`, not `Protocol`.
    // this is because `Protocol.lean` has several additional dependencies that we need to link with,
    // so we export it as a "Fat" static library.
    // see `lib/lakefile.lean` for more info.
    #[link(name = "ProtocolFat", kind = "static")]
    extern "C" {
        // https://doc.rust-lang.org/reference/items/external-blocks.html#the-link_name-attribute
        pub fn initialize_Protocol(
            builtin: u8,
            world: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;

        fn create_protocol(node_arr: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
        // TODO: remove if unused
        #[allow(dead_code)]
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
            round: usize,
        ) -> lean_sys::lean_obj_res;
        fn handle_message(
            p: lean_sys::lean_obj_arg,
            node_state: lean_sys::lean_obj_arg,
            src: lean_sys::lean_obj_arg,
            msg: lean_sys::lean_obj_arg,
        ) -> lean_sys::lean_obj_res;
        fn check_output(
            node_state: lean_sys::lean_obj_arg,
            leader: lean_sys::lean_obj_arg,
            round: usize,
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
        pub fn get_round(&self) -> usize {
            match &self {
                Self::InitialMsg { r, .. } | Self::EchoMsg { r, .. } | Self::VoteMsg { r, .. } => {
                    *r
                }
            }
        }
        pub unsafe fn from_lean(msg_lean: *mut lean_object) -> Self {
            // println!("[rb_protocol::lean::Message::from_lean] called");
            let tag = lean_ptr_tag(msg_lean);
            let mut current_field_id = 0;
            // let fields = lean_ctor_num_objs(msg_lean);

            // println!("[rb_protocol::lean::Message::from_lean] making assertions about tag {tag} with {fields} objects");

            // only EchoMsg and VoteMsg have the originator fields.
            let mut originator: String = String::new();
            if tag == 1 || tag == 2 {
                // println!("[rb_protocol::lean::Message::from_lean] deconstructing originator");
                // TODO: fix refcounting here too.
                originator =
                    lean_string_to_rust(lean_ctor_get(msg_lean, current_field_id), Mode::Borrow);
                current_field_id += 1;
            }

            // println!("[rb_protocol::lean::Message::from_lean] deconstructing round with id {current_field_id}");
            let rx = lean_ctor_get(msg_lean, current_field_id);
            // what_is_this("rx", rx);
            let r: usize = lean_unbox_usize(rx);
            current_field_id += 1;

            // println!("[rb_protocol::lean::Message::from_lean] deconstructing v with id {current_field_id}");
            // TODO: there is some dangling pointer issue here.
            // something about the way the packet list gets returned.
            // basically, it seems like `v` is freed after the first packet or something, so
            // we cannot use it again for the second packet?
            let vx = lean_ctor_get(msg_lean, current_field_id);
            // TODO: temporarily convert the lean string to rust as borrowed, since it's shared.
            // handle memory leaks later.
            let v = lean_string_to_rust(vx, Mode::Borrow);

            // TODO: re-free this.
            // right now, the same message is referenced across multiple packets.
            // so we can't free it.
            // figure out a way to make them all borrow from the packet somehow?
            // free the lean message
            // lean_dec(msg_lean);

            // println!("[rb_protocol::lean::Message::from_lean] done");
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
        pub src: String,
        pub dst: String,
        pub msg: Message,
        pub consumed: bool,
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
        /// Converts a Lean packet to its Rust representation.
        /// This function TAKES OWNERSHIP of the lean packet!
        /// Do NOT attempt to use the lean packet after calling this function.
        pub unsafe fn from_lean(packet_lean: *mut lean_object) -> Self {
            // println!("[rb_protocol::lean::Packet::from_lean] called");

            let src_lean = lean_ctor_get(packet_lean, 0);
            let dst_lean = lean_ctor_get(packet_lean, 1);
            let msg_lean = lean_ctor_get(packet_lean, 2);

            let consumed_lean_offset: std::ffi::c_uint =
                (3 * lean_helpers::VOID_PTR_SIZE).try_into().unwrap();
            let consumed_lean = lean_ctor_get_uint8(packet_lean, consumed_lean_offset);

            let src = lean_string_to_rust(src_lean, Mode::Owned);
            let dst = lean_string_to_rust(dst_lean, Mode::Owned);
            let msg = Message::from_lean(msg_lean);
            // no formal way to cast u8 to bool, so we do this instead
            let consumed: bool = consumed_lean != 0;

            // free the lean packet, since we have ownership
            // lean_dec(packet_lean);

            Packet {
                src,
                dst,
                msg,
                consumed,
            }
        }

        // Takes ownership of the rust packet.
        // note: doesn't seem like this is needed, since we can just convert the
        // message directly?
        // TODO: remove if unused
        #[allow(dead_code)]
        pub unsafe fn to_lean(self) -> *mut lean_object {
            create_packet(
                rust_string_to_lean(self.src),
                rust_string_to_lean(self.dst),
                Message::to_lean(self.msg),
                self.consumed as u8,
            )
        }
    }

    #[derive(Debug)]
    pub struct Protocol {
        pub protocol: *mut lean_object,
        pub node_state: *mut lean_object,
        pub round: usize,
        pub leader: String,
    }

    impl Protocol {
        pub unsafe fn create(node_list: Vec<String>, address: String, leader: String) -> Self {
            // initialize protocol
            let node_array_lean = lean_helpers::rust_string_vec_to_lean_array(node_list);
            let protocol = create_protocol(node_array_lean);

            // initialize this node's state
            let node_address_lean = rust_string_to_lean(address);
            // TODO: investigate lean reference-counting semantics
            //
            // it seems like calling `init_node_state` with `protocol` passes ownership of
            // `protocol` back to lean, resulting in a use-after-free segfault
            // if we continue to re-use `protocol` after this call (like we currently do).
            //
            // increment refcount here before calling so that lean doesn't automatically
            // free `protocol` once it's done initializing the node state.
            lean_inc(protocol);
            let node_state = init_node_state(protocol, node_address_lean);

            // initialize the global message hashtbl
            GLOBAL_MESSAGE_HASHTBL
                .set(Mutex::new(HashMap::new()))
                .expect("should be able to init global hashtbl");

            // initialize round to 0
            let round = 0;

            Protocol {
                protocol,
                node_state,
                round,
                leader,
            }
        }

        /// Deconstructs a Lean (new_state, packets_to_send) tuple into its Rust
        /// representation.
        /// This function TAKES OWNERSHIP of `state_and_packets`, and returns ownership of
        /// the new state and packet vector.
        unsafe fn deconstruct_state_and_packets(
            state_and_packets: *mut lean_object,
        ) -> (*mut lean_object, Vec<Packet>) {
            // println!("[rb_protocol::lean::deconstruct_state_and_packets] call");
            // note to self: investigate this part if anything goes wrong at runtime.

            // TODO: figure out the borrowing/ownership semantics here.
            // `state_and_packets` is a lean tuple. if i `lean_dec` the tuple, does
            // its fields get destructed too?
            //
            // i'm guessing that the tuple (product type) uses references to its contents,
            // so we can safely decrement its refcount without affecting its fields.
            // we're probably leaking memory all over the place, but we can worry about that later.

            // deconstruct new protocol state
            // println!(
            //     "[rb_protocol::lean::deconstruct_state_and_packets] deconstructing protocol state"
            // );
            assert!(lean_is_ctor(state_and_packets));
            assert!(lean_ctor_num_objs(state_and_packets) == 2);
            let new_state = lean_ctor_get(state_and_packets, 0);

            // deconstruct lean packets into rust
            // println!(
            //     "[rb_protocol::lean::deconstruct_state_and_packets] deconstructing lean packets"
            // );
            // RC: `lean_ctor_get` does not seem to increment the ref count.
            // we do not have to free `packets_arr_lean` later.
            let packets_arr_lean = lean_ctor_get(state_and_packets, 1);
            assert!(lean_is_array(packets_arr_lean));

            let n_packets: usize = lean_array_size(packets_arr_lean);

            let mut packets_to_send = Vec::new();
            for i in 0..n_packets {
                // println!(
                //     "[rb_protocol::lean::deconstruct_state_and_packets] deconstructing packet {i}"
                // );
                let packet_lean = lean_array_uget(packets_arr_lean, i); // borrows the lean packet

                // unmarshall the packet into rust.
                // note: Packet::from_lean takes ownership of the packet.
                let packet_rust = Packet::from_lean(packet_lean);
                packets_to_send.push(packet_rust);
            }

            // TODO: investigate the reference counting semantics
            // i think we might not actually have to decrement rc of the packet array,
            // because `lean_ctor_get` doesn't seem to increase the reference count?
            // not sure why, either.

            // RC: incrementing refcount of `new_state`.
            // i think we have to do this, since `lean_ctor_get` doesn't seem to increment it.
            // we don't want `new_state` to be freed when the result tuple gets freed.
            // println!("[rb_protocol::lean::deconstruct_state_and_packets] doing refcount stuff");
            lean_inc(new_state);

            // RC: decrement refcount of the result tuple
            // TODO: apparently decrementing this will segfault?
            // not sure how that works, but okay.
            // lean_dec(state_and_packets);

            (new_state, packets_to_send)
        }

        pub unsafe fn send_message(&mut self, address: String, message: String) -> Vec<Packet> {
            // println!("[rb_protocol::lean::send_message] {address} : {message} ");
            // update the message db with the current message
            let mut ht = GLOBAL_MESSAGE_HASHTBL
                .get()
                .expect("expected global message db to be initialized")
                .lock()
                .unwrap();

            ht.insert(address, message);
            // need to release the mutex on the global hashtable so that
            // `get_node_value` can acquire it.`
            std::mem::drop(ht);

            // send the InitialMessage
            // RC: increment refcount of `protocol`, since passing it into `send_message` gives it ownership,
            // and we need it to persist after the function call.
            // we do NOT increment `node_state`, since we no longer need it after the call.
            // println!("[rb_protocol::lean::send_message] call");
            lean_inc(self.protocol);
            let state_and_packets = send_message(self.protocol, self.node_state, self.round);

            let (new_state, packets_to_send) =
                Self::deconstruct_state_and_packets(state_and_packets);

            // update node state
            self.node_state = new_state;

            // increment round
            // note: this is maintained per-node for now, but eventually we may want some way
            // to broadcast the fact that we're starting a new round to all nodes.
            self.round += 1;

            packets_to_send
        }

        pub unsafe fn handle_packet(&mut self, packet: Packet) -> Vec<Packet> {
            // println!("[rb_protocol::lean::Protocol::handle_packet] called");
            let src_lean = rust_string_to_lean(packet.src);
            let msg_lean = Message::to_lean(packet.msg);

            // println!("[rb_protocol::lean::Protocol::handle_packet] calling handle_message in lean");
            // RC: increment refcount of `protocol`, since passing it into `send_message` gives it ownership,
            // and we need it to persist after the function call.
            // we do NOT increment `node_state`, since we no longer need it after the call.
            // likewise for `src_lean` and `msg_lean`
            lean_inc(self.protocol);
            let state_and_packets =
                handle_message(self.protocol, self.node_state, src_lean, msg_lean);

            // println!(
            //     "[rb_protocol::lean::Protocol::handle_packet] deconstructing state and packets"
            // );
            let (new_state, packets_to_send) =
                Self::deconstruct_state_and_packets(state_and_packets);

            // update node state
            self.node_state = new_state;

            // println!("[rb_protocol::lean::Protocol::handle_packet] returning");
            packets_to_send
        }

        pub unsafe fn check_output(&mut self, round: usize) {
            let leader = rust_string_to_lean(self.leader.clone());

            lean_inc(self.node_state);
            let _output_opt_lean = check_output(self.node_state, leader, round);

            // TODO: there's currently something very wrong with this, where
            // the result of the `check_output` call doesn't even seem to be a valid Lean object.
            // trying to do anything wiht it just segfaults.
            // currently, we just debug print the output from lean directly as a band-aid solution.
            // what_is_this("my option", output_opt_lean);

            // let cast = |lean_str| lean_string_to_rust(lean_str, Mode::Borrow);
            // let output_opt = lean_option_to_rust(output_opt_lean, cast);

            // // we would normally return [output_opt] here to pass back to the application code,
            // // but for now we just display it.
            // match output_opt {
            //     Some(v) => {
            //         println!("\n============ CONSENSUS OBTAINED FOR ROUND {round} =============");
            //         println!("\nValue: {v}\n");
            //         println!("===============================================================\n");
            //     }
            //     None => (),
            // }
        }
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
