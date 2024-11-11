# pb-rust

among other things, some kind of a implementation for provable broadcast (PB) in rust

## usage

### networktest

the poorly-named `networktest` module consists of 3 examples, in increasing complexity:

- `libp2p_mdns` ("m"): an example showcasing p2p peer discovery via mDNS
- `libp2p_mdns_ping` ("mp"): an example where new peers are automatically pinged when discovered via mDNS
- `libp2p_mdns_request_response` ("mrr"): an example showcasing a basic chatroom, where messages are sent via the `request_response` protocol and peer discovery via mDNS.
  - this example is also modular, in that we can easily convert this into a network driver for a custom protocol like PB by swapping out a few functions.

to run, execute `cargo run -- <example>`.
e.g. `cargo run -- mrr` to execute the `libp2p_mdns_request_response` example.

### reliable broadcast

its not working and it wont build yet until i figure out how to export the `lean-sts` dependency. wip

to get it to work just comment out the entire `rb_protocol.rs` and `libp2p_rb.rs` file and it should build fine, since those are the only parts that (implicitly) depend on lean-sts
