use crate::ffitest::lean_helpers;
use crate::networktest::rb_protocol;
use futures::prelude::*;
use libp2p::identity::Keypair;
use libp2p::request_response::{ProtocolSupport, ResponseChannel};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{mdns, request_response, PeerId, StreamProtocol, Swarm};
use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

use super::rb_protocol::{RBRequest, RBResponse};

fn truncate_peer_id(peer_id: &PeerId) -> String {
    let peer_id_string_ = peer_id.to_string();
    let peer_id_as_str = peer_id_string_.as_str();
    let truncated_peer_id_str = peer_id_as_str[peer_id_as_str.len() - 6..].to_string();
    truncated_peer_id_str
}

// define a custom behaviour, aggregating:
// - mdns behaviour for peer discovery
// - request_response behaviour for sending messages
//   - cbor as serialization mechanism
//   - <RBRequest, RBResponse> as the request and response type respectively
#[derive(NetworkBehaviour)]
struct RequestResponseMDNSBehaviour {
    mdns: mdns::tokio::Behaviour,
    request_response:
        request_response::cbor::Behaviour<rb_protocol::RBRequest, rb_protocol::RBResponse>,
}

impl RequestResponseMDNSBehaviour {
    fn new(keypair: &Keypair) -> Self {
        let local_peer_id = keypair.public().to_peer_id();
        let mdns_config = mdns::Config {
            ttl: Duration::from_secs(30),
            query_interval: Duration::from_secs(5),
            enable_ipv6: false,
        };
        Self {
            mdns: mdns::tokio::Behaviour::new(mdns_config, local_peer_id).unwrap(),
            request_response: request_response::cbor::Behaviour::<RBRequest, RBResponse>::new(
                [(
                    StreamProtocol::new("/verse-lab/reliable-broadcast/1"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
        }
    }
}

// request and response handlers
fn send_packet(
    swarm: &mut Swarm<RequestResponseMDNSBehaviour>,
    protocol: &mut rb_protocol::lean::Protocol,
    packet: rb_protocol::lean::Packet,
) {
    // note: `request_response::send_request` will automatically dial a peer
    // to send a message to them, if we don't yet have an active connection to them.
    let dst_id =
        PeerId::from_str(packet.dst.as_str()).expect("expected well-formed destination address");

    let self_id = swarm.local_peer_id();

    // libp2p does not support sending
    match &dst_id == self_id {
        false => {
            println!("sending packet to external destination:");
            dbg!(&packet);
            swarm
                .behaviour_mut()
                .request_response
                .send_request(&dst_id, RBRequest { packet });
        }
        true => {
            println!("sending packet to self:");
            dbg!(&packet);

            let packets_to_send = unsafe { protocol.handle_packet(packet) };
            dbg!(&packets_to_send);
            packets_to_send
                .into_iter()
                .for_each(|packet| send_packet(swarm, protocol, packet));
        }
    }
}

fn handle_request(
    swarm: &mut Swarm<RequestResponseMDNSBehaviour>,
    request: RBRequest,
    channel: ResponseChannel<RBResponse>,
    protocol: &mut rb_protocol::lean::Protocol,
) {
    // acknowledge the packet
    let response = RBResponse::Ack;
    swarm
        .behaviour_mut()
        .request_response
        .send_response(channel, response)
        .expect("should be able to ack a request");

    println!("received request:");
    dbg!(&request.packet);
    // generate new packets to send, and broadcast them
    let packets_to_send = unsafe { protocol.handle_packet(request.packet) };
    dbg!(&packets_to_send);

    packets_to_send
        .into_iter()
        .for_each(|packet| send_packet(swarm, protocol, packet));
}

fn handle_response(peer_id: &PeerId, response: &rb_protocol::RBResponse) {
    // in PB, this is where we would update the partial signature and generate a combined signature.
    // we would likely also be calling lean functions here.
    let truncated_peer_id = truncate_peer_id(peer_id);
    println!("{truncated_peer_id}: {response}");
}

// stdin is used for 2 different things.
// 1) if the protocol hasn't yet been initialized, sending "init" will be used to
// initialize the protocol using a snapshot of the current network state
// (i.e., create a protocol with all current nodes in the network)
//
// 2) if the protocol has been initialized, sending any message (including "init")
// will cause us to send that message to all other nodes using the given protocol.
fn handle_stdin(
    swarm: &mut Swarm<RequestResponseMDNSBehaviour>,
    line: &str,
    // a mutable reference might not be correct here.
    // may want to do something like a Box<T>? not sure
    protocol: &mut Option<rb_protocol::lean::Protocol>,
) {
    let my_address = swarm.local_peer_id().to_string();
    match (&protocol, line) {
        (None, "init") => {
            // initialize lean & protocol
            unsafe {
                lean_helpers::initialize_lean_environment(rb_protocol::lean::initialize_Protocol);

                let mut all_peers: Vec<String> =
                    swarm.connected_peers().map(PeerId::to_string).collect();
                all_peers.push(my_address.clone());
                dbg!(&all_peers);

                let new_protocol = rb_protocol::lean::Protocol::create(all_peers, my_address);

                protocol.replace(new_protocol);

                println!(">> initialized!")
            }
        }
        (None, _) => {
            // do nothing. before the protocol is initialized, we only accept the "init" command.
            println!(">> not yet initialized. run the 'init' command first!")
        }
        (Some(_), message) => {
            // generates packets to send from lean,
            let p = protocol.as_mut().unwrap();
            dbg!(&p);

            let packets_to_send = unsafe { p.send_message(my_address, String::from(message)) };

            // ..., then send them via libp2p.
            println!("[libp2p_rb::handle_stdin] sending packets");
            packets_to_send
                .into_iter()
                .for_each(|packet| send_packet(swarm, p, packet));
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // this was in the ping tutorial and i don't really know what it's for (yet)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // set up p2p network
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::tls::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|keypair| RequestResponseMDNSBehaviour::new(keypair))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))) // Allows us to observe pings indefinitely.
        .build();

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let my_truncated_peer_id = truncate_peer_id(swarm.local_peer_id());
    println!("my peer id: {my_truncated_peer_id}");

    // stdin reader
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // reliable broadcast protocol
    // TODO: maybe replace this option with a OnceCell?
    let mut protocol = None;

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
              handle_stdin(&mut swarm, &line, &mut protocol);
            }

            // handle a swarm event (poll the swarm)
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
                // MDNS: new peer discovered
                SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::Mdns(
                    mdns::Event::Discovered(list),
                )) => {
                    for (peer_id, _multiaddr) in list {
                        // upon discovery of new peer, dial them
                        let truncated_peer_id = truncate_peer_id(&peer_id);
                        println!("mdns discovered a new peer: {truncated_peer_id}");
                        swarm.dial(peer_id)?;
                    }
                }
                // MDNS: peer expired
                // TODO: determine what happens to the protocol when a peer expires and re-connects.
                SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::Mdns(
                    mdns::Event::Expired(list),
                )) => {
                    for (peer_id, _multiaddr) in list {
                        let truncated_peer_id = truncate_peer_id(&peer_id);
                        println!("mdns discover peer has expired: {truncated_peer_id}");
                    }
                }
                // Request-Response: received a request
                SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::RequestResponse(
                    request_response::Event::Message {
                        message:
                            request_response::Message::Request {
                                request,
                                channel,
                                ..
                            },
                            ..
                    },
                )) => {
                    handle_request(&mut swarm,  request, channel, protocol.as_mut().expect("protocol should be initialized"));
                }
                // Request-Response: received a response
                SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::RequestResponse(
                    request_response::Event::Message {
                        peer,
                        message:
                            request_response::Message::Response {
                                request_id: _,
                                response,
                            },
                    },
                )) => {
                    handle_response(&peer,  &response);
                }
                // Ignore all other events.
                _ => {}
            }
        }
    }
}
