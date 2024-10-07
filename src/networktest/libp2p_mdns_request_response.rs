use futures::prelude::*;
use libp2p::identity::Keypair;
use libp2p::request_response::ProtocolSupport;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{mdns, request_response, StreamProtocol};
use std::error::Error;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

// define a custom behaviour, aggregating:
// - mdns behaviour for peer discovery
// - request_response behaviour for sending messages
//   - cbor as serialization mechanism
//   - <String, String> as the request and response type respectively
#[derive(NetworkBehaviour)]
struct RequestResponseMDNSBehaviour {
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::cbor::Behaviour<String, String>,
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
            request_response: request_response::cbor::Behaviour::<String, String>::new(
                [(
                    StreamProtocol::new("/verse-lab/mdns-request-response-test/1"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // this was in the ping tutorial and i don't really know what it's for (yet)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

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

    let id = swarm.local_peer_id();
    println!("my peer id: {id}");

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
            // mdns handlers adapted from "chat" example in libp2p
            // https://github.com/libp2p/rust-libp2p/tree/master/examples/chat
            SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::Mdns(
                mdns::Event::Discovered(list),
            )) => {
                for (peer_id, _multiaddr) in list {
                    println!("mdns discovered a new peer: {peer_id}");
                    let request = format!("hello, {peer_id}!");
                    // note: `request_response::send_request` will automatically dial a peer
                    // to send a message to them, if we don't yet have an active connection to them.
                    swarm
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer_id, request);
                }
            }
            SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::Mdns(
                mdns::Event::Expired(list),
            )) => {
                for (peer_id, _multiaddr) in list {
                    println!("mdns discover peer has expired: {peer_id}");
                }
            }
            // handle a request
            SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    peer,
                    message:
                        request_response::Message::<String, String>::Request {
                            request_id,
                            request,
                            channel,
                        },
                },
            )) => {
                // sample response to a request
                println!("request from {peer} with id {request_id}: {request}");
                println!("sending response 1...");
                let response = format!("thank you for your request! here is response 1, {peer}.");
                swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, response)?;
            }
            // handle a response
            SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    peer,
                    message:
                        request_response::Message::<String, String>::Response {
                            request_id,
                            response,
                        },
                },
            )) => {
                // sample request acknowledgement
                println!("response from {peer} with id {request_id}: {response}");
            }
            _ => {}
        }
    }
}
