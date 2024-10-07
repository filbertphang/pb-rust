use futures::prelude::*;
use libp2p::mdns;
use libp2p::swarm::SwarmEvent;
use std::error::Error;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

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
        .with_behaviour(|keypair| {
            mdns::tokio::Behaviour::new(
                // set a custom config for mdns discovery.
                //
                // TTL just determines how long until the discover peer expires
                // this does not mean the peer is dead, it just means that it has to be re-discovered
                // by mdns again.
                // a dialed peer will remain connected even if the discovery expires.
                //
                // query interval determines how often mdns will try to discover new peers.
                mdns::Config {
                    ttl: Duration::from_secs(10),
                    query_interval: Duration::from_secs(5),
                    enable_ipv6: false,
                },
                keypair.public().to_peer_id(),
            )
            .unwrap()
        })?
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
            SwarmEvent::Behaviour(mdns::Event::Discovered(list)) => {
                for (peer_id, _multiaddr) in list {
                    println!("mdns discovered a new peer {peer_id} at {_multiaddr}");
                    swarm.dial(_multiaddr.clone())?;
                    println!("dialed {peer_id} at {_multiaddr}");
                }
            }
            SwarmEvent::Behaviour(mdns::Event::Expired(list)) => {
                for (peer_id, _multiaddr) in list {
                    println!("mdns discover peer {peer_id} at {_multiaddr} has expired");
                }
            }
            _ => {}
        }
    }
}
