use futures::prelude::*;
use libp2p::identity::Keypair;
use libp2p::request_response::{ProtocolSupport, RequestId, ResponseChannel};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{mdns, request_response, PeerId, StreamProtocol, Swarm};
use std::error::Error;
use std::fmt::Display;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

fn truncate_peer_id(peer_id: &PeerId) -> String {
    let peer_id_string_ = peer_id.to_string();
    let peer_id_as_str = peer_id_string_.as_str();
    let truncated_peer_id_str = peer_id_as_str[peer_id_as_str.len() - 6..].to_string();
    truncated_peer_id_str
}

// define a custom request and response type.
// in PB, this would likely be the `Packet` type.
// we only need to derive `serde::{Serialize, Deserialize}``, then use
// one of the `request_response` serializers (like `cbor` or `json`).
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct MyRequestType {
    msg: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct MyResponseType {
    msg: String,
}

// may want to consider using a crate like `derive_more` to help us derive
// `Display` here.
impl Display for MyRequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request: {}", self.msg)
    }
}

impl Display for MyResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "response: {}", self.msg)
    }
}

impl Error for MyResponseType {}

// define a custom behaviour, aggregating:
// - mdns behaviour for peer discovery
// - request_response behaviour for sending messages
//   - cbor as serialization mechanism
//   - <MyRequestType, MyResponseType> as the request and response type respectively
#[derive(NetworkBehaviour)]
struct RequestResponseMDNSBehaviour {
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::cbor::Behaviour<MyRequestType, MyResponseType>,
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
            request_response:
                request_response::cbor::Behaviour::<MyRequestType, MyResponseType>::new(
                    [(
                        StreamProtocol::new("/verse-lab/mdns-request-response-test/1"),
                        ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
        }
    }
}

// request and response handlers
fn send_init_request(swarm: &mut Swarm<RequestResponseMDNSBehaviour>, peer_id: &PeerId) {
    let truncated_peer_id = truncate_peer_id(peer_id);
    let request = MyRequestType {
        msg: format!("hello, {truncated_peer_id}!"),
    };
    // note: `request_response::send_request` will automatically dial a peer
    // to send a message to them, if we don't yet have an active connection to them.
    swarm
        .behaviour_mut()
        .request_response
        .send_request(peer_id, request);
}

fn handle_request(
    swarm: &mut Swarm<RequestResponseMDNSBehaviour>,
    peer_id: &PeerId,
    request_id: &RequestId,
    request: &MyRequestType,
    channel: ResponseChannel<MyResponseType>,
) -> Result<(), MyResponseType> {
    // in PB, this is where we would read a packet, update the state, and determine
    // what packets to return.
    // we would likely be calling lean functions here.
    let truncated_peer_id = truncate_peer_id(peer_id);
    println!("request from {truncated_peer_id} with id {request_id}: {request}");
    println!("sending response 1...");
    let response = MyResponseType {
        msg: format!("thank you for your request! here is response 1, {truncated_peer_id}."),
    };
    swarm
        .behaviour_mut()
        .request_response
        .send_response(channel, response)
}

fn handle_response(peer_id: &PeerId, request_id: &RequestId, response: &MyResponseType) {
    // in PB, this is where we would update the partial signature and generate a combined signature.
    // we would likely also be calling lean functions here.
    let truncated_peer_id = truncate_peer_id(peer_id);
    println!("response from {truncated_peer_id} with id {request_id}: {response}");
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

    let my_truncated_peer_id = truncate_peer_id(swarm.local_peer_id());
    println!("my peer id: {my_truncated_peer_id}");

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
            // MDNS: new peer discovered
            SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::Mdns(
                mdns::Event::Discovered(list),
            )) => {
                for (peer_id, _multiaddr) in list {
                    let truncated_peer_id = truncate_peer_id(&peer_id);
                    println!("mdns discovered a new peer: {truncated_peer_id}");
                    send_init_request(&mut swarm, &peer_id);
                }
            }
            // MDNS: peer expired
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
                    peer,
                    message:
                        request_response::Message::Request {
                            request_id,
                            request,
                            channel,
                        },
                },
            )) => {
                handle_request(&mut swarm, &peer, &request_id, &request, channel)?;
            }
            // Request-Response: received a response
            SwarmEvent::Behaviour(RequestResponseMDNSBehaviourEvent::RequestResponse(
                request_response::Event::Message {
                    peer,
                    message:
                        request_response::Message::Response {
                            request_id,
                            response,
                        },
                },
            )) => {
                handle_response(&peer, &request_id, &response);
            }
            // Ignore all other events.
            _ => {}
        }
    }
}
