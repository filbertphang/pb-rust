mod networktest {
    pub mod libp2p_mdns_ping;
    pub mod tcp;
}

fn main() {
    let _ = networktest::libp2p_mdns_ping::main();
}
