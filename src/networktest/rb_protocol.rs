use crate::ffitest::lean_helpers;
pub mod lean {
    use crate::ffitest::lean_helpers::{self, rust_string_to_lean};

    #[link(name = "Protocol")]
    extern "C" {
        // https://doc.rust-lang.org/reference/items/external-blocks.html#the-link_name-attribute
        #[link_name = "initialize_Protocol"]
        pub fn initialize(builtin: u8, world: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;

        fn create_protocol(node_arr: lean_sys::lean_obj_arg) -> lean_sys::lean_obj_res;
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

    pub struct Protocol {
        protocol: *mut lean_object,
        node_state: *mut lean_object,
    }

    impl Protocol {
        pub unsafe fn create(node_list: Vec<String>, address: String) -> Protocol {
            // initialize protocol
            let node_list_raw: Vec<usize> = node_list.into_iter().map(|s| {
                let c_str_s = std::ffi::CString::new(s).unwrap();
                c_str_s.as_ptr() as usize
            });
            let node_array_lean = lean_helpers::rust_usize_vec_to_lean_array(node_list_raw);
            let protocol = create_protocol(node_array_lean);

            // initialize this node's state
            let node_address_lean = rust_string_to_lean(address);
            let node_state = init_node_state(protocol, node_address_lean);

            Protocol {
                protocol,
                node_state,
            }
        }
    }
}

// define a custom request and response type.
// in PB, this would likely be the `Packet` type.
// we only need to derive `serde::{Serialize, Deserialize}``, then use
// one of the `request_response` serializers (like `cbor` or `json`).
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct MyRequestType {
    msg: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct MyResponseType {
    msg: String,
}

// may want to consider using a crate like `derive_more` to help us derive
// `Display` here.
impl Display for MyRequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request: {}", self.msg)
    }
}

impl Display for MyResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "response: {}", self.msg)
    }
}

impl Error for MyResponseType {}
