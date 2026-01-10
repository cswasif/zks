//! ZKS MCP Server - AI-powered post-quantum development
//! 
//! This crate provides a Model Context Protocol (MCP) server for the ZKS Protocol,
//! enabling AI agents to interact with post-quantum cryptography, anonymous networking,
//! and secure communications.

use std::path::PathBuf;

pub mod tools;
pub mod resources;
pub mod prompts;
pub mod transport;

pub use tools::{CryptoTools, NetworkTools, DevTools, TestTools, AnalysisTools};
pub use resources::ZksResourceProvider;

/// Main ZKS MCP Server implementation
#[derive(Clone)]
pub struct ZksMcpServer {
    zks_protocol_root: PathBuf,
    crypto_tools: CryptoTools,
    network_tools: NetworkTools,
    dev_tools: DevTools,
    test_tools: TestTools,
    analysis_tools: AnalysisTools,
    resource_provider: ZksResourceProvider,
}

impl ZksMcpServer {
    pub fn new() -> Self {
        let zks_protocol_root = PathBuf::from(".");
        Self {
            zks_protocol_root: zks_protocol_root.clone(),
            crypto_tools: CryptoTools::new(),
            network_tools: NetworkTools::new(),
            dev_tools: DevTools::new(),
            test_tools: TestTools::new(),
            analysis_tools: AnalysisTools::new(),
            resource_provider: ZksResourceProvider::new(zks_protocol_root.clone()),

        }
    }

    pub fn with_zks_protocol_root<P: Into<PathBuf>>(mut self, root: P) -> Self {
        let zks_protocol_root = root.into();
        self.zks_protocol_root = zks_protocol_root.clone();
        self.resource_provider = ZksResourceProvider::new(zks_protocol_root);
        self
    }

    pub fn build(self) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(self)
    }

    pub fn crypto_tools(&self) -> &CryptoTools {
        &self.crypto_tools
    }

    pub fn network_tools(&self) -> &NetworkTools {
        &self.network_tools
    }

    pub fn dev_tools(&self) -> &DevTools {
        &self.dev_tools
    }

    pub fn test_tools(&self) -> &TestTools {
        &self.test_tools
    }

    pub fn analysis_tools(&self) -> &AnalysisTools {
        &self.analysis_tools
    }

    pub fn resource_provider(&self) -> &ZksResourceProvider {
        &self.resource_provider
    }


}

impl Default for ZksMcpServer {
    fn default() -> Self {
        Self::new()
    }
}