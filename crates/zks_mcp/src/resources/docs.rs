//! Documentation resources for ZKS MCP server
//! 
//! Provides access to ZKS Protocol documentation, API docs, security guides,
//! and architecture documentation.

use rmcp::model::{ResourceTemplate, ResourceContents, RawResourceTemplate};
use rmcp::ErrorData;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Clone)]
pub struct DocsResource {
    zks_protocol_root: PathBuf,
}

impl DocsResource {
    pub fn new<P: Into<PathBuf>>(zks_protocol_root: P) -> Self {
        Self {
            zks_protocol_root: zks_protocol_root.into(),
        }
    }

    pub fn resources(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://docs/readme".into(),
                    name: "README Documentation".into(),
                    title: None,
                    description: Some("Main project documentation".into()),
                    mime_type: Some("text/markdown".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://docs/crates/{crate_name}".into(),
                    name: "Crate Documentation".into(),
                    title: None,
                    description: Some("Documentation for specific ZKS crate".into()),
                    mime_type: Some("text/markdown".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://docs/api/{crate}/{module}".into(),
                    name: "API Documentation".into(),
                    title: None,
                    description: Some("API documentation for ZKS modules".into()),
                    mime_type: Some("text/markdown".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://docs/security".into(),
                    name: "Security Documentation".into(),
                    title: None,
                    description: Some("ZKS Protocol security documentation".into()),
                    mime_type: Some("text/markdown".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://docs/architecture".into(),
                    name: "Architecture Documentation".into(),
                    title: None,
                    description: Some("ZKS Protocol architecture overview".into()),
                    mime_type: Some("text/markdown".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://docs/protocols/{protocol}".into(),
                    name: "Protocol Documentation".into(),
                    title: None,
                    description: Some("ZKS Protocol specifications".into()),
                    mime_type: Some("text/markdown".into()),
                },
                annotations: None,
            },
        ]
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContents, ErrorData> {
        let path = self.resolve_doc_path(uri)?;
        
        if !path.exists() {
            return Err(ErrorData::resource_not_found(format!("Documentation not found: {}", uri), None));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ErrorData::internal_error(format!("Failed to read documentation: {}", e), None))?;

        Ok(ResourceContents::TextResourceContents {
            uri: uri.to_string(),
            mime_type: Some("text/markdown".to_string()),
            text: content,
            meta: None,
        })
    }

    fn resolve_doc_path(&self, uri: &str) -> Result<PathBuf, ErrorData> {
        let parts: Vec<&str> = uri.strip_prefix("zks://docs/").unwrap_or(uri).split('/').collect();
        
        match parts.as_slice() {
            ["readme"] => Ok(self.zks_protocol_root.join("README.md")),
            ["crates", crate_name] => {
                let crate_path = self.zks_protocol_root.join("crates").join(crate_name);
                let readme_path = crate_path.join("README.md");
                if readme_path.exists() {
                    Ok(readme_path)
                } else {
                    // Try to generate basic crate documentation
                    Ok(self.generate_crate_docs(crate_name)?)
                }
            },
            ["api", crate_name, module] => {
                let doc_path = self.zks_protocol_root.join("target").join("doc").join(crate_name).join(format!("{}.html", module));
                if doc_path.exists() {
                    // Convert HTML to markdown for better AI consumption
                    Ok(self.convert_html_to_markdown(&doc_path)?)
                } else {
                    Ok(self.generate_api_docs(crate_name, module)?)
                }
            },
            ["security"] => Ok(self.zks_protocol_root.join("docs").join("SECURITY.md")),
            ["architecture"] => Ok(self.zks_protocol_root.join("docs").join("ARCHITECTURE.md")),
            ["protocols", protocol] => {
                match *protocol {
                    "zk" => Ok(self.zks_protocol_root.join("docs").join("protocols").join("ZK_PROTOCOL.md")),
                    "zks" => Ok(self.zks_protocol_root.join("docs").join("protocols").join("ZKS_PROTOCOL.md")),
                    _ => Err(ErrorData::resource_not_found(format!("Unknown protocol: {}", protocol), None)),
                }
            },
            _ => Err(ErrorData::resource_not_found(format!("Unknown documentation path: {}", uri), None)),
        }
    }

    fn generate_crate_docs(&self, crate_name: &str) -> Result<PathBuf, ErrorData> {
        let crate_path = self.zks_protocol_root.join("crates").join(crate_name);
        let cargo_toml = crate_path.join("Cargo.toml");
        
        if !cargo_toml.exists() {
            return Err(ErrorData::resource_not_found(format!("Crate not found: {}", crate_name), None));
        }

        // Generate basic crate documentation
        let doc_content = format!(
            "# {}\n\n\
            This is the {} crate from the ZKS Protocol.\n\n\
            ## Overview\n\n\
            The {} crate provides core functionality for the ZKS Protocol.\n\n\
            ## Usage\n\n\
            Add this to your `Cargo.toml`:\n\n\
            ```toml\n\
            [{}]\n\
            path = \"{}\"\n\
            ```\n\n\
            ## Documentation\n\n\
            For detailed API documentation, see `zks://docs/api/{}/`.\n",
            crate_name, crate_name, crate_name, crate_name, crate_path.display(), crate_name
        );

        let doc_path = crate_path.join("README.md");
        fs::write(&doc_path, doc_content)
            .map_err(|e| ErrorData::internal_error(format!("Failed to write crate docs: {}", e), None))?;

        Ok(doc_path)
    }

    fn generate_api_docs(&self, crate_name: &str, module: &str) -> Result<PathBuf, ErrorData> {
        let doc_path = self.zks_protocol_root.join("target").join("doc").join(crate_name).join(format!("{}.md", module));
        
        let api_content = format!(
            "# {}::{} API Documentation\n\n\
            This is the API documentation for the `{}` module in the `{}` crate.\n\n\
            ## Module Overview\n\n\
            The `{}` module provides essential functionality for the ZKS Protocol.\n\n\
            ## Functions\n\n\
            For function-level documentation, please refer to the source code or run:\n\n\
            ```bash\n\
            cargo doc --open --package {}\n\
            ```\n",
            crate_name, module, module, crate_name, module, crate_name
        );

        fs::write(&doc_path, api_content)
            .map_err(|e| ErrorData::internal_error(format!("Failed to write API docs: {}", e), None))?;

        Ok(doc_path)
    }

    fn convert_html_to_markdown(&self, html_path: &Path) -> Result<PathBuf, ErrorData> {
        // For now, just return the HTML path - in a real implementation,
        // we would convert HTML to markdown using a library like html2md
        Ok(html_path.to_path_buf())
    }
}

impl Default for DocsResource {
    fn default() -> Self {
        Self::new(".")
    }
}