use std::env::args;

mod networktest {
    pub mod libp2p_mdns;
    pub mod libp2p_mdns_ping;
    pub mod libp2p_mdns_request_response;
    pub mod tcp;
}

pub mod ffitest;

fn main() {
    let _ = match args().nth(1).unwrap().as_str() {
        "m" => networktest::libp2p_mdns::main().unwrap(),
        "mp" => networktest::libp2p_mdns_ping::main().unwrap(),
        "mrr" => networktest::libp2p_mdns_request_response::main().unwrap(),
        "ffi" => {
            let module = args().nth(2).unwrap();
            ffitest::main(module.as_str());
        }
        e => panic!("please enter a module to execute (invalid module: {e})"),
    };
}
