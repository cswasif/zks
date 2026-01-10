//! Unified SwarmController for zks:// onion routing
//! 
//! This module provides a platform-agnostic interface that automatically
//! detects the runtime environment (Native vs WASM) and uses the appropriate
//! transport layer for onion routing.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

#[cfg(not(target_arch = "wasm32"))]
use crate::p2p::NativeP2PTransport;
#[cfg(not(target_arch = "wasm32"))]
use crate::signaling::SignalingClient;

#[cfg(target_arch = "wasm32")]
use crate::signaling::SignalingClient;

/// Platform detection and transport selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Native,
    WebAssembly,
}

impl Platform {
    /// Detect the current platform at runtime
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Platform::WebAssembly
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Platform::Native
        }
    }
}

/// Unified swarm controller that automatically selects the appropriate transport
pub struct SwarmController {
    platform: Platform,
    signaling_client: Arc<RwLock<Option<SignalingClient>>>,
    
    #[cfg(not(target_arch = "wasm32"))]
    native_transport: Arc<RwLock<Option<NativeP2PTransport>>>,
    
    is_connected: Arc<RwLock<bool>>,
    local_peer_id: Arc<RwLock<Option<String>>>,
}

impl SwarmController {
    /// Create a new swarm controller
    pub async fn new() -> Result<Self, SwarmControllerError> {
        let platform = Platform::detect();
        info!("Initializing SwarmController for platform: {:?}", platform);
        
        Ok(Self {
            platform,
            signaling_client: Arc::new(RwLock::new(None)),
            
            #[cfg(not(target_arch = "wasm32"))]
            native_transport: Arc::new(RwLock::new(None)),
            
            is_connected: Arc::new(RwLock::new(false)),
            local_peer_id: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Get the current platform
    pub fn platform(&self) -> Platform {
        self.platform
    }
    
    /// Connect to the swarm using the appropriate transport
    pub async fn connect(
        &self,
        signaling_url: &str,
        local_peer_id: String,
    ) -> Result<(), SwarmControllerError> {
        debug!("Connecting to swarm via signaling server: {}", signaling_url);
        
        // Store local peer ID
        *self.local_peer_id.write().await = Some(local_peer_id.clone());
        
        // Create and connect signaling client
        let signaling_client = SignalingClient::connect(signaling_url, local_peer_id).await
            .map_err(|e| SwarmControllerError::SignalingError(format!("Failed to connect to signaling server: {}", e)))?;
        
        *self.signaling_client.write().await = Some(signaling_client);
        *self.is_connected.write().await = true;
        
        info!("Successfully connected to swarm via signaling server");
        Ok(())
    }
    
    /// Join a swarm room for peer discovery
    pub async fn join_room(&self, room_id: &str, capabilities: crate::signaling::PeerCapabilities) -> Result<(), SwarmControllerError> {
        if let Some(client) = self.signaling_client.write().await.as_mut() {
            client.join_room(room_id, capabilities).await
                .map_err(|e| SwarmControllerError::SignalingError(format!("Failed to join room: {}", e)))?;
            
            info!("Joined swarm room: {}", room_id);
            Ok(())
        } else {
            Err(SwarmControllerError::NotConnected)
        }
    }
    
    /// Discover peers in the current room
    pub async fn discover_peers(&self, room_id: &str) -> Result<Vec<crate::signaling::PeerInfo>, SwarmControllerError> {
        if let Some(client) = self.signaling_client.write().await.as_mut() {
            let peers = client.discover_peers(room_id).await
                .map_err(|e| SwarmControllerError::SignalingError(format!("Failed to discover peers: {}", e)))?;
            
            debug!("Discovered {} peers in room {}", peers.len(), room_id);
            Ok(peers)
        } else {
            Err(SwarmControllerError::NotConnected)
        }
    }
    
    /// Get swarm entropy for cryptographic operations
    pub async fn get_swarm_entropy(&self, room_id: &str) -> Result<[u8; 32], SwarmControllerError> {
        if let Some(client) = self.signaling_client.write().await.as_mut() {
            let entropy = client.get_swarm_entropy(room_id).await
                .map_err(|e| SwarmControllerError::SignalingError(format!("Failed to get swarm entropy: {}", e)))?;
            
            debug!("Retrieved {} bytes of swarm entropy", entropy.len());
            Ok(entropy)
        } else {
            Err(SwarmControllerError::NotConnected)
        }
    }
    
    /// Get the local peer ID
    pub async fn local_peer_id(&self) -> Option<String> {
        self.local_peer_id.read().await.clone()
    }
    
    /// Check if connected to the swarm
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }
    
    /// Disconnect from the swarm
    pub async fn disconnect(&self) -> Result<(), SwarmControllerError> {
        if let Some(_client) = self.signaling_client.write().await.take() {
            // Client will be dropped, which closes the connection
            info!("Disconnected from swarm");
        }
        
        *self.is_connected.write().await = false;
        Ok(())
    }
    
    /// Get platform-specific transport capabilities
    pub fn transport_capabilities(&self) -> TransportCapabilities {
        match self.platform {
            Platform::Native => TransportCapabilities {
                supports_direct_p2p: true,
                supports_nat_traversal: true,
                supports_relay: true,
                max_hops: 8,
                min_hops: 2,
            },
            Platform::WebAssembly => TransportCapabilities {
                supports_direct_p2p: false,
                supports_nat_traversal: false,
                supports_relay: true,
                max_hops: 6,
                min_hops: 3,
            },
        }
    }
    
    /// Build an onion circuit for the specified number of hops
    pub async fn build_onion_circuit(&self, target_peer: &str, min_hops: u8, max_hops: u8) -> Result<String, SwarmControllerError> {
        let capabilities = self.transport_capabilities();
        
        if min_hops < capabilities.min_hops || max_hops > capabilities.max_hops {
            return Err(SwarmControllerError::InvalidCircuitConfig(format!(
                "Hops must be between {} and {}",
                capabilities.min_hops,
                capabilities.max_hops
            )));
        }
        
        // For now, we'll use a simple approach: select random peers from the room
        // In a full implementation, this would involve complex path selection algorithms
        
        let room_id = "default"; // TODO: Get from configuration
        let peers = self.discover_peers(room_id).await?;
        
        if peers.len() < (max_hops as usize - 1) {
            return Err(SwarmControllerError::NotEnoughPeers(format!(
                "Need at least {} peers for {}-hop circuit, found {}",
                max_hops - 1,
                max_hops,
                peers.len()
            )));
        }
        
        // Select random peers for the circuit
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut selected_peers = peers.clone();
        selected_peers.shuffle(&mut rng);
        
        let target_peer_info = crate::signaling::PeerInfo {
            peer_id: target_peer.to_string(),
            public_key: vec![],
            capabilities: crate::signaling::PeerCapabilities::default(),
            last_seen: 0,
            addresses: vec![],
        };
        
        let circuit_peers: Vec<_> = selected_peers
            .iter()
            .take((max_hops - 1) as usize)
            .chain(std::iter::once(&target_peer_info))
            .collect();
        
        info!("Building {}-hop onion circuit to {} via {} peers", max_hops, target_peer, circuit_peers.len() - 1);
        
        // Generate circuit ID
        let circuit_id = format!("circuit_{}", uuid::Uuid::new_v4());
        
        // For WASM, we would use the browser onion transport
        // For native, we would use direct P2P connections
        // This is a simplified implementation
        
        Ok(circuit_id)
    }
    
    /// Send data through an established onion circuit
    pub async fn send_through_circuit(&self, circuit_id: &str, data: &[u8]) -> Result<(), SwarmControllerError> {
        // This would implement the actual onion routing protocol
        // For now, this is a placeholder
        debug!("Would send {} bytes through circuit {}", data.len(), circuit_id);
        Ok(())
    }
    
    /// Receive data from any circuit
    pub async fn receive_from_circuit(&self, circuit_id: &str) -> Result<Option<Vec<u8>>, SwarmControllerError> {
        // This would implement receiving data through the onion circuit
        // For now, this is a placeholder
        debug!("Would receive data from circuit {}", circuit_id);
        Ok(None)
    }
    
    /// Tear down an onion circuit
    pub async fn teardown_circuit(&self, circuit_id: &str) -> Result<(), SwarmControllerError> {
        info!("Tearing down circuit {}", circuit_id);
        // This would implement circuit teardown
        Ok(())
    }
    
    /// Create an onion stream that routes through the specified circuit
    pub async fn create_onion_stream(&self, circuit_id: &str) -> Result<OnionStream, SwarmControllerError> {
        info!("Creating onion stream for circuit {}", circuit_id);
        
        // For now, create a basic onion stream
        // In a full implementation, this would establish the actual routing through the circuit
        Ok(OnionStream::new(circuit_id.to_string()))
    }
}

/// Transport capabilities for different platforms
#[derive(Debug, Clone)]
pub struct TransportCapabilities {
    pub supports_direct_p2p: bool,
    pub supports_nat_traversal: bool,
    pub supports_relay: bool,
    pub max_hops: u8,
    pub min_hops: u8,
}

/// Errors that can occur in the swarm controller
#[derive(Debug, thiserror::Error)]
pub enum SwarmControllerError {
    #[error("Not connected to swarm")]
    NotConnected,
    
    #[error("Signaling error: {0}")]
    SignalingError(String),
    
    #[error("Transport error: {0}")]
    TransportError(String),
    
    #[error("Invalid circuit configuration: {0}")]
    InvalidCircuitConfig(String),
    
    #[error("Not enough peers available: {0}")]
    NotEnoughPeers(String),
    
    #[error("Circuit error: {0}")]
    CircuitError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// An onion routing stream that routes data through an established circuit
pub struct OnionStream {
    circuit_id: String,
    read_buffer: std::collections::VecDeque<u8>,
    write_buffer: std::collections::VecDeque<u8>,
}

impl OnionStream {
    /// Create a new onion stream for the specified circuit
    pub fn new(circuit_id: String) -> Self {
        Self {
            circuit_id,
            read_buffer: std::collections::VecDeque::new(),
            write_buffer: std::collections::VecDeque::new(),
        }
    }
    
    /// Get the circuit ID this stream is associated with
    pub fn circuit_id(&self) -> &str {
        &self.circuit_id
    }
}

impl tokio::io::AsyncRead for OnionStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let n = std::cmp::min(buf.remaining(), self.read_buffer.len());
        if n > 0 {
            let data: Vec<u8> = self.read_buffer.drain(..n).collect();
            buf.put_slice(&data);
        }
        std::task::Poll::Ready(Ok(()))
    }
}

impl tokio::io::AsyncWrite for OnionStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        self.write_buffer.extend(buf);
        std::task::Poll::Ready(Ok(buf.len()))
    }
    
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
    
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_platform_detection() {
        let platform = Platform::detect();
        
        #[cfg(target_arch = "wasm32")]
        assert_eq!(platform, Platform::WebAssembly);
        
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(platform, Platform::Native);
    }
    
    #[tokio::test]
    async fn test_swarm_controller_creation() {
        let controller = SwarmController::new().await.unwrap();
        assert!(controller.is_connected().await == false);
    }
    
    #[tokio::test]
    async fn test_transport_capabilities() {
        let controller = SwarmController::new().await.unwrap();
        let capabilities = controller.transport_capabilities();
        
        match controller.platform() {
            Platform::Native => {
                assert!(capabilities.supports_direct_p2p);
                assert!(capabilities.supports_nat_traversal);
                assert!(capabilities.max_hops >= 6);
            }
            Platform::WebAssembly => {
                assert!(!capabilities.supports_direct_p2p);
                assert!(capabilities.supports_relay);
                assert!(capabilities.max_hops <= 6);
            }
        }
    }
}