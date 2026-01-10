//! Native P2P transport using libp2p for desktop/mobile platforms
//! 
//! This module provides full peer-to-peer networking capabilities for native
//! platforms (not WASM), including:
//! - Direct TCP/UDP connections
//! - NAT traversal with DCUtR (Direct Connection Upgrade through Relay)
//! - Full libp2p protocol support
//! - Hole punching capabilities

#[cfg(not(target_arch = "wasm32"))]
use libp2p::{
    identity::Keypair,
    swarm::{SwarmEvent, Swarm},
    tcp::{tokio::Transport as TcpTransport, Config as TcpConfig},
    noise,
    yamux,
    relay,
    dcutr,
    ping,
    PeerId,
    Multiaddr,
    Transport,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn, error};
use futures_util::StreamExt;

/// Custom event type for NativeSwarmBehaviour
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum NativeSwarmEvent {
    Ping(ping::Event),
    Relay(relay::Event),
    Dcutr(dcutr::Event),
}

impl From<ping::Event> for NativeSwarmEvent {
    fn from(event: ping::Event) -> Self {
        NativeSwarmEvent::Ping(event)
    }
}

impl From<relay::Event> for NativeSwarmEvent {
    fn from(event: relay::Event) -> Self {
        NativeSwarmEvent::Relay(event)
    }
}

impl From<dcutr::Event> for NativeSwarmEvent {
    fn from(event: dcutr::Event) -> Self {
        NativeSwarmEvent::Dcutr(event)
    }
}

/// Native P2P swarm behavior combining all necessary protocols
#[cfg(not(target_arch = "wasm32"))]
#[derive(libp2p::swarm::NetworkBehaviour)]
#[behaviour(to_swarm = "NativeSwarmEvent")]
struct NativeSwarmBehaviour {
    /// Ping protocol for connectivity testing
    ping: ping::Behaviour,
    /// Relay protocol for NAT traversal
    relay: relay::Behaviour,
    /// DCUtR protocol for hole punching
    dcutr: dcutr::Behaviour,
}

/// Native P2P transport for desktop/mobile platforms
#[cfg(not(target_arch = "wasm32"))]
pub struct NativeP2PTransport {
    swarm: Swarm<NativeSwarmBehaviour>,
    local_peer_id: PeerId,
    connected_peers: Arc<Mutex<HashMap<PeerId, Vec<Multiaddr>>>>,
    event_receiver: mpsc::UnboundedReceiver<SwarmEvent<NativeSwarmEvent>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeP2PTransport {
    /// Create a new native P2P transport
    pub async fn new(keypair: Option<Keypair>) -> Result<Self, NativeP2PError> {
        let keypair = keypair.unwrap_or_else(Keypair::generate_ed25519);
        let local_peer_id = PeerId::from(keypair.public());
        
        info!("Creating native P2P transport with peer ID: {}", local_peer_id);
        
        // Create transport with TCP, noise, and yamux
        let transport = TcpTransport::new(TcpConfig::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&keypair).map_err(|e| NativeP2PError::Noise(e.to_string()))?)
            .multiplex(yamux::Config::default())
            .boxed();
        
        // Create swarm behavior
        let behaviour = NativeSwarmBehaviour {
            ping: ping::Behaviour::new(ping::Config::new()),
            relay: relay::Behaviour::new(local_peer_id, Default::default()),
            dcutr: dcutr::Behaviour::new(local_peer_id),
        };
        
        // Create swarm
        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                TcpConfig::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .build();
        
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            swarm,
            local_peer_id,
            connected_peers: Arc::new(Mutex::new(HashMap::new())),
            event_receiver,
        })
    }
    
    /// Listen on a local address
    pub async fn listen_on(&mut self, addr: Multiaddr) -> Result<(), NativeP2PError> {
        self.swarm.listen_on(addr)?;
        info!("Native P2P transport listening on swarm addresses");
        Ok(())
    }
    
    /// Dial a peer at the given address
    pub async fn dial(&mut self, peer_addr: Multiaddr) -> Result<(), NativeP2PError> {
        info!("Dialing peer at: {}", peer_addr);
        self.swarm.dial(peer_addr)?;
        Ok(())
    }
    
    /// Get the local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }
    
    /// Get swarm addresses
    pub fn listen_addresses(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }
    
    /// Start the event loop
    pub async fn run(mut self) -> Result<(), NativeP2PError> {
        info!("Starting native P2P transport event loop");
        
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {}", address);
                }
                SwarmEvent::Behaviour(event) => {
                    match event {
                        event => {
                             debug!("Unhandled swarm behaviour event: {:?}", event);
                         }
                    }
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    info!("Connected to {} via {}", peer_id, endpoint.get_remote_address());
                    
                    let mut peers = self.connected_peers.lock().await;
                    peers.entry(peer_id).or_default().push(endpoint.get_remote_address().clone());
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    warn!("Connection closed to {}: {:?}", peer_id, cause);
                    
                    let mut peers = self.connected_peers.lock().await;
                    peers.remove(&peer_id);
                }
                SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                    debug!("Incoming connection from {} to {}", send_back_addr, local_addr);
                }
                SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                    error!("Incoming connection error from {} to {}: {}", send_back_addr, local_addr, error);
                }
                SwarmEvent::Dialing { peer_id, .. } => {
                    debug!("Dialing peer {:?}", peer_id);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    error!("Outgoing connection error to {:?}: {}", peer_id, error);
                }
                _ => {}
            }
        }
    }
    
    /// Get connected peers
    pub async fn connected_peers(&self) -> Vec<PeerId> {
        let peers = self.connected_peers.lock().await;
        peers.keys().cloned().collect()
    }
    
    /// Check if connected to a specific peer
    pub async fn is_connected(&self, peer_id: &PeerId) -> bool {
        let peers = self.connected_peers.lock().await;
        peers.contains_key(peer_id)
    }
}

/// Errors that can occur in native P2P transport
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, thiserror::Error)]
pub enum NativeP2PError {
    #[error("Transport error: {0}")]
    Transport(#[from] libp2p::TransportError<std::io::Error>),
    
    #[error("Swarm error: {0}")]
    Swarm(String),
    
    #[error("Dial error: {0}")]
    Dial(String),
    
    #[error("Dial error: {0}")]
    DialError(#[from] libp2p::swarm::DialError),
    
    #[error("Noise error: {0}")]
    NoiseError(#[from] libp2p::noise::Error),
    
    #[error("Infallible error")]
    Infallible(#[from] std::convert::Infallible),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Noise error: {0}")]
    Noise(String),
}

/// Stub implementation for WASM targets
#[cfg(target_arch = "wasm32")]
pub struct NativeP2PTransport;

#[cfg(target_arch = "wasm32")]
impl NativeP2PTransport {
    pub async fn new(_keypair: Option<()>) -> Result<Self, String> {
        Err("Native P2P transport not available in WASM".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[cfg(not(target_arch = "wasm32"))]
    async fn test_native_p2p_creation() {
        let transport = NativeP2PTransport::new(None).await.unwrap();
        let peer_id = transport.local_peer_id();
        assert!(!peer_id.to_string().is_empty());
    }
}