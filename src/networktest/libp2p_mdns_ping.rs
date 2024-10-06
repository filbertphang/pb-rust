use futures::prelude::*;
use libp2p::identity::Keypair;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{mdns, ping};
use std::error::Error;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

// define a custom behaviour, aggregating:
// - mdns behaviour for peer discovery
// - ping behaviour as the actual protocol we're using
// (next step will be to test mdns + request_response protocol)
#[derive(NetworkBehaviour)]
struct PingMDNSBehaviour {
    mdns: mdns::tokio::Behaviour,
    ping: ping::Behaviour,
}

impl PingMDNSBehaviour {
    fn new(keypair: &Keypair) -> Self {
        let local_peer_id = keypair.public().to_peer_id();
        Self {
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id).unwrap(),
            ping: ping::Behaviour::default(),
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
        .with_behaviour(|keypair| PingMDNSBehaviour::new(keypair))?
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
            SwarmEvent::Behaviour(PingMDNSBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mdns discovered a new peer: {peer_id}");
                    swarm.dial(_multiaddr.clone())?;
                    println!("dialed {peer_id} at {_multiaddr}");
                }
            }
            SwarmEvent::Behaviour(PingMDNSBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mdns discover peer has expired: {peer_id}");
                }
            }
            // ping handler adapted from ping tutorial
            // https://docs.rs/libp2p/latest/libp2p/tutorials/ping/index.html
            SwarmEvent::Behaviour(PingMDNSBehaviourEvent::Ping(event)) => {
                println!("ping event: {event:?}");
            }
            _ => {}
        }
    }
}
