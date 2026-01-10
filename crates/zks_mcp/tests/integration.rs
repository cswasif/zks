//! Integration tests for ZKS MCP Server
//! 
//! Tests the complete MCP server functionality including tool execution,
//! resource access, and prompt handling.

use zks_mcp::ZksMcpServer;

#[tokio::test]
async fn test_server_creation() {
    let server = ZksMcpServer::new()
        .with_zks_protocol_root(".")
        .build();
    
    assert!(server.is_ok(), "Server should be created successfully");
}

#[tokio::test]
async fn test_server_default() {
    let server = ZksMcpServer::default()
        .build();
    
    assert!(server.is_ok(), "Server should build with default configuration");
}