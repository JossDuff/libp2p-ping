use futures::prelude::*;
use libp2p::{ping, swarm::SwarmEvent, Multiaddr};
use std::error::Error;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Need a Transport and NetworkBehaviour to build a swarm.
    // A Swarm drives both a Transport and a NetworkBehaviour forward, passing commands from the
    // NetworkBehaviour to the Transport, as well as events from the Transport to the
    // NetworkBehaviour.
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_async_std()
        // tcp is the Transport type.  This defines HOW to send bytes on the network
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::tls::Config::new,
            libp2p::yamux::Config::default,
        )?
        // ping is the NetworkBehaviour type.  This defines WHAT bytes and WHOM to send them to.
        .with_behaviour(|_| ping::Behaviour::default())?
        // for this example we need to set the idle connection timeout, otherwise the connection
        // will be close immediately.  Typically, connections are kept alive if they are "in use"
        // by a certain protocol, but the ping protocol is only an "auxiliary" kind of protocol.
        // Without any other behaviour in place, we wouldn't be able to observe the pings.
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    // tell the swarm to listen on all interfaces and a random, OS-assigned port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second command line argument, if
    // any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

    // drive the Swarm coninuously, allowing it to listen for incoming connections and establish an
    // outgoing connection in case we specify an address on the cli.
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
            _ => {}
        }
    }
}
