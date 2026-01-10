use zks_mcp::ZksMcpServer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting ZKS MCP Server...");
    
    let _server = ZksMcpServer::new()
        .with_zks_protocol_root(".")
        .build()?;
    
    // For now just exit, as transport not implemented
    // server.serve(stdio()).await?;
    
    Ok(())
}