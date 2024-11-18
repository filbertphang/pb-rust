use crate::ffitest::lean_helpers::*;
use crate::networktest::rb_protocol;
use lean_sys::*;

#[no_mangle]
pub unsafe extern "C" fn dbg_print_rust(s: *mut lean_object) -> usize {
    let ss = lean_string_to_rust(s, Mode::Owned);
    println!("[from lean]: {ss}");

    return 0;
}
pub fn main() {
    unsafe {
        initialize_lean_environment(rb_protocol::lean::initialize_Protocol);

        let peer = String::from("peer_addr");
        let peer2 = String::from("peer2_addr");
        let me = String::from("my_addr");

        println!("creating protocol...");
        let mut my_protocol =
            rb_protocol::lean::Protocol::create(vec![peer.clone(), peer2.clone()], me.clone());

        let my_init_text = String::from("this is an initial message");
        let init_packets = my_protocol.send_message(me.clone(), my_init_text);
        dbg!(&init_packets);

        // let my_echo_text = String::from("this is an echo message");
        // let my_echo_msg = rb_protocol::lean::Message::EchoMsg {
        //     originator: peer.clone(),
        //     r: 1,
        //     v: my_echo_text,
        // };
        // let my_echo_packet = Packet {
        //     src: peer.clone(),
        //     dst: me.clone(),
        //     msg: my_echo_msg,
        //     consumed: false,
        // };
        // println!("echo packets:");
        // dbg!(&my_echo_packet);

        // let outputs = my_protocol.handle_packet(my_echo_packet);
        // dbg!(&outputs);

        // let my_echo2_text = String::from("this is an echo2 message");
        // let my_echo2_msg = rb_protocol::lean::Message::EchoMsg {
        //     originator: peer.clone(),
        //     r: 2,
        //     v: my_echo2_text,
        // };
        // let my_echo2_packet = Packet {
        //     src: peer.clone(),
        //     dst: me.clone(),
        //     msg: my_echo2_msg,
        //     consumed: false,
        // };
        // println!("echo2 packets:");
        // dbg!(&my_echo2_packet);

        // let outputs = my_protocol.handle_packet(my_echo2_packet);
        // dbg!(&outputs);
    }

    println!("sadge");
}
