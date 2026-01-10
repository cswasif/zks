//! Resource providers for ZKS MCP server
//! 
//! This module provides resource templates and implementations for accessing
//! ZKS Protocol documentation, code, examples, and status information.

pub mod code;
pub mod docs;
pub mod examples;
pub mod status;

pub use code::CodeResources;
pub use docs::DocsResource;
pub use examples::ExamplesResource;
pub use status::StatusResource;

use rmcp::model::{ResourceTemplate, ResourceContents, ErrorData};
use std::sync::Arc;

/// Combined resource provider that handles all ZKS resources
#[derive(Clone)]
pub struct ZksResourceProvider {
    docs: Arc<DocsResource>,
    code: Arc<CodeResources>,
    examples: Arc<ExamplesResource>,
    status: Arc<StatusResource>,
}

impl ZksResourceProvider {
    pub fn new(zks_protocol_root: std::path::PathBuf) -> Self {
        Self {
            docs: Arc::new(DocsResource::new(zks_protocol_root.clone())),
            code: Arc::new(CodeResources::new(zks_protocol_root.clone())),
            examples: Arc::new(ExamplesResource::new(zks_protocol_root.to_string_lossy().into_owned())),
            status: Arc::new(StatusResource::new(zks_protocol_root.to_string_lossy().into_owned())),
        }
    }

    pub fn resources(&self) -> Vec<ResourceTemplate> {
        let mut all_resources = Vec::new();
        
        // Add documentation resources
        all_resources.extend(self.docs.resources());
        
        // Add code resources
        all_resources.extend(self.code.resources());
        
        // Add example resources
        all_resources.extend(self.examples.resources());
        
        // Add status resources
        all_resources.extend(self.status.resources());
        
        all_resources
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContents, ErrorData> {
        // Route the request to the appropriate resource provider
        if uri.starts_with("zks://docs/") {
            self.docs.read_resource(uri).await
        } else if uri.starts_with("zks://code/") {
            self.code.read_resource(uri).await
        } else if uri.starts_with("zks://examples/") {
            self.examples.read_resource(uri).await
        } else if uri.starts_with("zks://status/") {
            self.status.read_resource(uri).await
        } else {
            Err(ErrorData::resource_not_found(format!("Unknown resource URI: {}", uri), None))
        }
    }
}

impl Default for ZksResourceProvider {
    fn default() -> Self {
        Self::new(std::path::PathBuf::from("."))
    }
}