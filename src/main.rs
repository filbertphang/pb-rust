use std::env::args;

mod networktest {
    pub mod libp2p_mdns;
    pub mod libp2p_mdns_ping;
    pub mod libp2p_mdns_request_response;
    pub mod tcp;
}

fn main() {
    let _ = match args().nth(1).unwrap().as_str() {
        "m" => networktest::libp2p_mdns::main(),
        "mp" => networktest::libp2p_mdns_ping::main(),
        "mrr" => networktest::libp2p_mdns_request_response::main(),
        e => panic!("please enter a module to execute (invalid module: {e})"),
    };
}
