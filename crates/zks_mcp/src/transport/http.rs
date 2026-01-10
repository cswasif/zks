//! HTTP transport implementation for ZKS MCP server
//! 
//! Provides HTTP transport for remote AI agents with authentication support.

use rmcp::transport::streamable_http_server;

#[derive(Clone)]
pub struct ZksHttpTransport;

impl ZksHttpTransport {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZksHttpTransport {
    fn default() -> Self {
        Self::new()
    }
}