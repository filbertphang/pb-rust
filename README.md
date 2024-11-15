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

latest update: 15 Nov 24 1636H

- build: ok
- runs without crashing with 2 nodes total (1 leader 1 echoer)
  - fails for more than 2 nodes, due to the dangling message pointer issue (see below)
- protocol is not correct
  - need to include self in the node list when creating protocol
  - need to handle self messaging since we can't swarm dial self
  - need to handle the dangling message pointer issue for packets (search for TODO)
