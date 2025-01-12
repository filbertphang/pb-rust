# lean-rust-examples

a very messy sandbox containing various examples for working with lean structures in rust (and vice versa)

(as of 12/01/25: i may decide to tidy this up eventually, but it's not a priority right now!)

## usage

### ffitest

self-explanatory: the various files (`arrays`, `globals`, `simple`, `structs`) showcase simple examples of working with those types of structures in lean.

the complementary lean code can be found in `lib/`.

### networktest

**this section has nothing to do with lean!**

the poorly-named `networktest` module consists of 3 examples, in increasing complexity:

- `libp2p_mdns` ("m"): an example showcasing p2p peer discovery via mDNS
- `libp2p_mdns_ping` ("mp"): an example where new peers are automatically pinged when discovered via mDNS
- `libp2p_mdns_request_response` ("mrr"): an example showcasing a basic chatroom, where messages are sent via the `request_response` protocol and peer discovery via mDNS.
  - this example is also modular, in that we can easily convert this into a network driver for a custom protocol like PB by swapping out a few functions.

to run, execute `cargo run -- <example>`.
e.g. `cargo run -- mrr` to execute the `libp2p_mdns_request_response` example.

#### reliable broadcast

latest update: 19 Nov 24 0207H

- initial proof of concept

**install**

- no special install instructions. just make sure you have lean toolchain v4.11.0 installed in elan, cargo
  should settle the rust dependencies automatically

**instructions to run**

- git clone the repo
- open `n` terminal windows
- in all of them, run `cargo run -- rb`
- ensure that all nodes have discovered each other. you should see a message like the following for each of the `n-1` other nodes: `mdns discovered a new peer: FJLDJE`
- pick a node as the leader node. you should see its peer id printed like:
  `my peer id: 12D3KooWFgPsALZhvdDneAhRukLg92BrNSAXhpZV18nj9r9g3vm7`
- copy the peer id, and type `init <leader-peer-id>` into each terminal window. all of them should say `>> initialized`.
- in the leader node, type anything and press enter. this will be treated as the message, and will be broadcast to all nodes
- watch as the nodes achieve consensus!
