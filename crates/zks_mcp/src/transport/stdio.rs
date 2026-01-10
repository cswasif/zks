//! Stdio transport implementation for ZKS MCP server
//! 
//! Provides stdio transport for local AI assistants (Claude Desktop, VS Code, etc.).

use rmcp::transport::stdio;

#[derive(Clone)]
pub struct ZksStdioTransport;

impl ZksStdioTransport {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZksStdioTransport {
    fn default() -> Self {
        Self::new()
    }
}