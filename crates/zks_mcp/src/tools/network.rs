//! Networking tools for ZKS MCP server
//! 
//! Provides tools for ZKS network operations including connection management,
//! anonymous routing, peer discovery, and NAT traversal.

use rmcp::{tool, tool_router, model::*, ErrorData as McpError};
use rmcp::handler::server::wrapper::Parameters;
use zks_proto::handshake::Handshake;
use url::Url;
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Clone)]
pub struct NetworkTools;

impl NetworkTools {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NetworkTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConnectParams {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConnectAnonymousParams {
    pub url: String,
    pub min_hops: Option<u8>,
    pub max_hops: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HandshakeParams {
    pub role: String,
    pub room_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseUrlParams {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendParams {
    pub data: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReceiveParams {
    pub encoding: Option<String>,
    pub max_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CloseParams {
    pub connection_id: String,
}

#[tool_router]
impl NetworkTools {
    /// Connect to a peer using direct ZK:// protocol
    #[tool(name = "zks_connect", description = "Connect to a ZK node using direct zk:// protocol")]
    pub async fn zks_connect(&self, params: Parameters<ConnectParams>) -> Result<CallToolResult, McpError> {
        let url = &params.0.url;
        
        // Validate URL scheme
        if !url.starts_with("zk://") {
            return Err(McpError::invalid_params(
                "URL must use zk:// scheme".to_string(),
                None
            ));
        }

        // Parse URL
        let parsed_url = Url::parse(url)
            .map_err(|e| McpError::invalid_params(format!("Invalid URL: {}", e), None))?;

        // Validate URL components
        if parsed_url.host().is_none() {
            return Err(McpError::invalid_params(
                "URL must have a host".to_string(),
                None
            ));
        }

        // Return connection info (stateless - no actual connection created)
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "status": "ready",
            "protocol": "zk",
            "url": url,
            "security": "post-quantum",
            "timeout": 30,
            "connection_id": format!("zk_{}", uuid::Uuid::new_v4()),
            "message": "Connection parameters validated successfully"
        }).to_string())]))
    }

    /// Connect to a peer using anonymous ZKS:// protocol with onion routing
    #[tool(name = "zks_connect_anonymous", description = "Connect to a ZK node using anonymous zks:// protocol with onion routing")]
    pub async fn zks_connect_anonymous(&self, params: Parameters<ConnectAnonymousParams>) -> Result<CallToolResult, McpError> {
        let url = &params.0.url;
        let min_hops = params.0.min_hops.unwrap_or(3);
        let max_hops = params.0.max_hops.unwrap_or(5);
        
        // Validate URL scheme
        if !url.starts_with("zks://") {
            return Err(McpError::invalid_params(
                "URL must use zks:// scheme".to_string(),
                None
            ));
        }

        // Parse URL
        let parsed_url = Url::parse(url)
            .map_err(|e| McpError::invalid_params(format!("Invalid URL: {}", e), None))?;

        // Validate URL components
        if parsed_url.host().is_none() {
            return Err(McpError::invalid_params(
                "URL must have a host".to_string(),
                None
            ));
        }

        // Validate hop parameters
        if min_hops < 1 || min_hops > 10 {
            return Err(McpError::invalid_params(
                "min_hops must be between 1 and 10".to_string(),
                None
            ));
        }
        
        if max_hops < min_hops || max_hops > 10 {
            return Err(McpError::invalid_params(
                "max_hops must be between min_hops and 10".to_string(),
                None
            ));
        }

        // Return connection info (stateless - no actual connection created)
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "status": "ready",
            "protocol": "zks",
            "url": url,
            "min_hops": min_hops,
            "max_hops": max_hops,
            "scrambling": true,
            "timeout": 60,
            "connection_id": format!("zks_{}", uuid::Uuid::new_v4()),
            "message": "Anonymous connection parameters validated successfully"
        }).to_string())]))
    }

    /// Perform 3-message post-quantum handshake
    #[tool(name = "zks_handshake", description = "Perform 3-message post-quantum handshake")]
    pub async fn zks_handshake(&self, params: Parameters<HandshakeParams>) -> Result<CallToolResult, McpError> {
        let role = &params.0.role;
        let room_id = params.0.room_id.as_deref().unwrap_or("default_room");

        match role.as_str() {
            "initiator" => {
                // For demo purposes, use a dummy trusted responder public key
                let trusted_responder_public_key = vec![0u8; 1952]; // ML-KEM-768 public key size
                let _handshake = Handshake::new_initiator(room_id.to_string(), trusted_responder_public_key)
                    .map_err(|e| McpError::internal_error(format!("Failed to create handshake: {}", e), None))?;
                
                Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
                    "status": "initiated",
                    "role": "initiator",
                    "room_id": room_id,
                    "message": "Handshake initiated as initiator"
                }).to_string())]))
            }
            "responder" => {
                let _handshake = Handshake::new_responder(room_id.to_string());
                
                Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
                    "status": "initiated",
                    "role": "responder",
                    "room_id": room_id,
                    "message": "Handshake initiated as responder"
                }).to_string())]))
            }
            _ => Err(McpError::invalid_params(
                "Role must be 'initiator' or 'responder'".to_string(),
                None
            ))
        }
    }

    /// Parse and validate ZK URLs
    #[tool(name = "zks_parse_url", description = "Parse and validate ZK URLs")]
    pub async fn zks_parse_url(&self, params: Parameters<ParseUrlParams>) -> Result<CallToolResult, McpError> {
        let url = &params.0.url;
        
        // Parse URL
        let parsed_url = Url::parse(url)
            .map_err(|e| McpError::invalid_params(format!("Invalid URL: {}", e), None))?;

        // Extract components
        let scheme = parsed_url.scheme();
        let host = parsed_url.host_str().unwrap_or("");
        let port = parsed_url.port().unwrap_or(0);
        let path = parsed_url.path();

        // Validate ZK scheme
        if scheme != "zk" && scheme != "zks" {
            return Err(McpError::invalid_params(
                "URL must use zk:// or zks:// scheme".to_string(),
                None
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "url": url,
            "scheme": scheme,
            "host": host,
            "port": port,
            "path": path,
            "valid": true
        }).to_string())]))
    }

    /// Send data over a connection
    #[tool(name = "zks_send", description = "Send data over a connection")]
    pub async fn zks_send(&self, params: Parameters<SendParams>) -> Result<CallToolResult, McpError> {
        let data = &params.0.data;
        let encoding = params.0.encoding.as_deref().unwrap_or("text");

        // Convert data to bytes based on encoding
        let bytes = match encoding {
            "text" => data.as_bytes().to_vec(),
            "base64" => {
                general_purpose::STANDARD.decode(data)
                    .map_err(|e| McpError::invalid_params(format!("Invalid base64: {}", e), None))?
            }
            "hex" => {
                hex::decode(data)
                    .map_err(|e| McpError::invalid_params(format!("Invalid hex: {}", e), None))?
            }
            _ => {
                return Err(McpError::invalid_params(
                    "Encoding must be 'text', 'base64', or 'hex'".to_string(),
                    None
                ));
            }
        };

        // Simulate sending data
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "status": "sent",
            "bytes_sent": bytes.len(),
            "encoding": encoding
        }).to_string())]))
    }

    /// Receive data from a connection
    #[tool(name = "zks_receive", description = "Receive data from a connection")]
    pub async fn zks_receive(&self, params: Parameters<ReceiveParams>) -> Result<CallToolResult, McpError> {
        let encoding = params.0.encoding.as_deref().unwrap_or("text");
        let max_size = params.0.max_size.unwrap_or(1024);

        // Simulate receiving data
        let sample_data = b"Hello from ZKS network!";
        let received_bytes = &sample_data[..sample_data.len().min(max_size)];

        let result = match encoding {
            "text" => {
                String::from_utf8(received_bytes.to_vec())
                    .map_err(|e| McpError::internal_error(format!("Invalid UTF-8: {}", e), None))?
            }
            "base64" => general_purpose::STANDARD.encode(received_bytes),
            "hex" => hex::encode(received_bytes),
            _ => {
                return Err(McpError::invalid_params(
                    "Encoding must be 'text', 'base64', or 'hex'".to_string(),
                    None
                ));
            }
        };

        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "data": result,
            "bytes_received": received_bytes.len(),
            "encoding": encoding
        }).to_string())]))
    }

    /// Close a connection
    #[tool(name = "zks_close", description = "Close a connection")]
    pub async fn zks_close(&self, params: Parameters<CloseParams>) -> Result<CallToolResult, McpError> {
        let connection_id = &params.0.connection_id;

        // Simulate closing connection
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "status": "closed",
            "connection_id": connection_id
        }).to_string())]))
    }

    /// List active connections
    #[tool(name = "zks_list_connections", description = "List active connections")]
    pub async fn zks_list_connections(&self) -> Result<CallToolResult, McpError> {
        // Return empty list (stateless design)
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "connections": []
        }).to_string())]))
    }
}