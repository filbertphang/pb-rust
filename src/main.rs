use std::env::args;

mod networktest {
    pub mod libp2p_mdns;
    pub mod libp2p_mdns_ping;
    pub mod libp2p_mdns_request_response;
    pub mod libp2p_rb;
    mod rb_protocol;
    #[allow(dead_code, unused_variables)]
    pub mod sandbox;
    #[allow(dead_code)]
    pub mod tcp;
}

mod ffitest {
    pub mod arrays;
    pub mod globals;
    pub mod lean_helpers;
    pub mod simple;
    pub mod structs;
}

fn main() {
    let _ = match args().nth(1).unwrap().as_str() {
        "m" => networktest::libp2p_mdns::main().unwrap(),
        "mp" => networktest::libp2p_mdns_ping::main().unwrap(),
        "mrr" => networktest::libp2p_mdns_request_response::main().unwrap(),
        "rb" => networktest::libp2p_rb::main().unwrap(),
        "sb" => networktest::sandbox::main(),
        "ffis" => {
            let module = args().nth(2).unwrap();
            ffitest::simple::main(module.as_str());
        }
        "ffig" => {
            let module = args().nth(2).unwrap();
            ffitest::globals::main(module.as_str());
        }
        "ffil" => {
            let module = args().nth(2).unwrap();
            ffitest::arrays::main(module.as_str());
        }
        "ffist" => {
            let module = args().nth(2).unwrap();
            ffitest::structs::main(module.as_str());
        }
        e => panic!("please enter a module to execute (invalid module: {e})"),
    };
}
