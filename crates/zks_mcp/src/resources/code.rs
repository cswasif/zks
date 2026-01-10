//! Code resources for ZKS MCP server
//! 
//! Provides access to ZKS Protocol source code, function definitions,
//! struct definitions, and implementation blocks.

use rmcp::model::{ResourceTemplate, ResourceContents, RawResourceTemplate};
use rmcp::ErrorData;
use std::fs;
use std::path::PathBuf;
use serde_json::json;

#[derive(Clone)]
pub struct CodeResources {
    zks_protocol_root: PathBuf,
}

impl CodeResources {
    pub fn new(zks_protocol_root: PathBuf) -> Self {
        Self {
            zks_protocol_root,
        }
    }

    pub fn resources(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://code/crate/{name}".into(),
                    name: "Crate Source".into(),
                    title: None,
                    description: Some("Crate source listing".into()),
                    mime_type: Some("application/json".into()),
                    },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://code/file/{path}".into(),
                    name: "File Contents".into(),
                    title: None,
                    description: Some("File source code".into()),
                    mime_type: Some("text/x-rust".into()),
                    },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://code/function/{crate}/{path}".into(),
                    name: "Function Definition".into(),
                    title: None,
                    description: Some("Function source code".into()),
                    mime_type: Some("text/x-rust".into()),
                    },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://code/struct/{crate}/{name}".into(),
                    name: "Struct Definition".into(),
                    title: None,
                    description: Some("Struct definition".into()),
                    mime_type: Some("text/x-rust".into()),
                    },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://code/impl/{crate}/{struct}".into(),
                    name: "Implementation Block".into(),
                    title: None,
                    description: Some("Implementation block".into()),
                    mime_type: Some("text/x-rust".into()),
                    },
                annotations: None,
            },
        ]
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContents, ErrorData> {
        if uri.starts_with("zks://code/crate/") {
            let crate_name = uri.strip_prefix("zks://code/crate/").unwrap();
            self.read_crate_source(crate_name).await
        } else if uri.starts_with("zks://code/file/") {
            let file_path = uri.strip_prefix("zks://code/file/").unwrap();
            self.read_file_contents(file_path).await
        } else if uri.starts_with("zks://code/function/") {
            let path = uri.strip_prefix("zks://code/function/").unwrap();
            let parts: Vec<&str> = path.splitn(2, '/').collect();
            if parts.len() != 2 {
                return Err(rmcp::ErrorData::invalid_params("Invalid function path", None));
            }
            self.read_function_definition(parts[0], parts[1]).await
        } else if uri.starts_with("zks://code/struct/") {
            let path = uri.strip_prefix("zks://code/struct/").unwrap();
            let parts: Vec<&str> = path.splitn(2, '/').collect();
            if parts.len() != 2 {
                return Err(rmcp::ErrorData::invalid_params("Invalid struct path", None));
            }
            self.read_struct_definition(parts[0], parts[1]).await
        } else if uri.starts_with("zks://code/impl/") {
            let path = uri.strip_prefix("zks://code/impl/").unwrap();
            let parts: Vec<&str> = path.splitn(2, '/').collect();
            if parts.len() != 2 {
                return Err(rmcp::ErrorData::invalid_params("Invalid impl path", None));
            }
            self.read_implementation_block(parts[0], parts[1]).await
        } else {
            Err(rmcp::ErrorData::resource_not_found(format!("Unknown resource URI: {}", uri), None))
        }
    }

    async fn read_crate_source(&self, crate_name: &str) -> Result<ResourceContents, rmcp::ErrorData> {
        let crate_path = self.zks_protocol_root.join("crates").join(crate_name);
        
        if !crate_path.exists() {
            return Err(rmcp::ErrorData::resource_not_found(format!("Crate not found: {}", crate_name), None));
        }

        let src_path = crate_path.join("src");
        let files = self.list_rust_files(&src_path)?;
        
        let content = json!({
            "crate": crate_name,
            "path": crate_path.to_string_lossy(),
            "files": files,
            "has_tests": crate_path.join("tests").exists(),
            "has_examples": crate_path.join("examples").exists(),
        });

        Ok(ResourceContents::TextResourceContents {
            uri: format!("zks://code/crate/{}", crate_name),
            mime_type: Some("application/json".to_string()),
            text: content.to_string(),
            meta: None,
        })
    }

    async fn read_file_contents(&self, file_path: &str) -> Result<ResourceContents, rmcp::ErrorData> {
        let full_path = self.zks_protocol_root.join(file_path);
        
        if !full_path.exists() {
            return Err(rmcp::ErrorData::resource_not_found(format!("File not found: {}", file_path), None));
        }

        let content = fs::read_to_string(&full_path)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read file: {}", e), None))?;

        Ok(ResourceContents::TextResourceContents {
            uri: format!("zks://code/file/{}", file_path),
            mime_type: Some("text/x-rust".to_string()),
            text: content,
            meta: None,
        })
    }

    async fn read_function_definition(&self, crate_name: &str, function_path: &str) -> Result<ResourceContents, rmcp::ErrorData> {
        let file_path = self.zks_protocol_root.join("crates").join(crate_name).join("src").join(function_path);
        
        if !file_path.exists() {
            return Err(rmcp::ErrorData::resource_not_found(format!("File not found: {}/{}", crate_name, function_path), None));
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read file: {}", e), None))?;

        // Simple function extraction - in a real implementation, you'd use a proper AST parser
        let function_code = self.extract_function(&content, function_path)?;

        Ok(ResourceContents::TextResourceContents {
            uri: format!("zks://code/function/{}/{}", crate_name, function_path),
            mime_type: Some("text/x-rust".to_string()),
            text: function_code,
            meta: None,
        })
    }

    async fn read_struct_definition(&self, crate_name: &str, struct_name: &str) -> Result<ResourceContents, rmcp::ErrorData> {
        let crate_path = self.zks_protocol_root.join("crates").join(crate_name);
        let src_path = crate_path.join("src");
        
        if !src_path.exists() {
            return Err(rmcp::ErrorData::resource_not_found(format!("Crate src not found: {}", crate_name), None));
        }

        // Find the struct definition
        let struct_code = self.find_struct_definition(&src_path, struct_name)?;

        Ok(ResourceContents::TextResourceContents {
            uri: format!("zks://code/struct/{}/{}", crate_name, struct_name),
            mime_type: Some("text/x-rust".to_string()),
            text: struct_code,
            meta: None,
        })
    }

    async fn read_implementation_block(&self, crate_name: &str, struct_name: &str) -> Result<ResourceContents, rmcp::ErrorData> {
        let crate_path = self.zks_protocol_root.join("crates").join(crate_name);
        let src_path = crate_path.join("src");
        
        if !src_path.exists() {
            return Err(rmcp::ErrorData::resource_not_found(format!("Crate src not found: {}", crate_name), None));
        }

        // Find the implementation block
        let impl_code = self.find_implementation_block(&src_path, struct_name)?;

        Ok(ResourceContents::TextResourceContents {
            uri: format!("zks://code/impl/{}/{}", crate_name, struct_name),
            mime_type: Some("text/x-rust".to_string()),
            text: impl_code,
            meta: None,
        })
    }

    fn list_rust_files(&self, dir: &PathBuf) -> Result<Vec<String>, rmcp::ErrorData> {
        let mut files = Vec::new();
        
        if dir.exists() {
            for entry in fs::read_dir(dir).map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read directory: {}", e), None))? {
                let entry = entry.map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read entry: {}", e), None))?;
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                    files.push(path.file_name().unwrap().to_string_lossy().to_string());
                } else if path.is_dir() {
                    let sub_files = self.list_rust_files(&path)?;
                    for file in sub_files {
                        files.push(format!("{}/{}", path.file_name().unwrap().to_string_lossy(), file));
                    }
                }
            }
        }
        
        Ok(files)
    }

    fn extract_function(&self, content: &str, _function_path: &str) -> Result<String, rmcp::ErrorData> {
        // Simple extraction - return the entire file content for now
        // In a real implementation, you'd parse the AST to extract specific functions
        Ok(content.to_string())
    }

    fn find_struct_definition(&self, src_path: &PathBuf, struct_name: &str) -> Result<String, rmcp::ErrorData> {
        // Search through all Rust files for the struct definition
        for entry in fs::read_dir(src_path).map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read directory: {}", e), None))? {
            let entry = entry.map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read entry: {}", e), None))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read file: {}", e), None))?;
                
                // Simple search for struct definition
                if content.contains(&format!("struct {}", struct_name)) || 
                   content.contains(&format!("pub struct {}", struct_name)) {
                    // Extract the struct definition (simplified)
                    let lines: Vec<&str> = content.lines().collect();
                    let mut struct_lines = Vec::new();
                    let mut in_struct = false;
                    let mut brace_count = 0;
                    
                    for line in lines {
                        if line.contains(&format!("struct {}", struct_name)) || 
                           line.contains(&format!("pub struct {}", struct_name)) {
                            in_struct = true;
                        }
                        
                        if in_struct {
                            struct_lines.push(line);
                            brace_count += line.matches('{').count();
                            brace_count -= line.matches('}').count();
                            
                            if brace_count == 0 && struct_lines.len() > 1 {
                                break;
                            }
                        }
                    }
                    
                    if !struct_lines.is_empty() {
                        return Ok(struct_lines.join("\n"));
                    }
                }
            }
        }
        
        Err(rmcp::ErrorData::resource_not_found(format!("Struct not found: {}", struct_name), None))
    }

    fn find_implementation_block(&self, src_path: &PathBuf, struct_name: &str) -> Result<String, rmcp::ErrorData> {
        // Search through all Rust files for the implementation block
        for entry in fs::read_dir(src_path).map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read directory: {}", e), None))? {
            let entry = entry.map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read entry: {}", e), None))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read file: {}", e), None))?;
                
                // Simple search for impl block
                if content.contains(&format!("impl {}", struct_name)) || 
                   content.contains(&format!("impl<")) && content.contains(struct_name) {
                    // Extract the impl block (simplified)
                    let lines: Vec<&str> = content.lines().collect();
                    let mut impl_lines = Vec::new();
                    let mut in_impl = false;
                    let mut brace_count = 0;
                    
                    for line in lines {
                        if line.contains(&format!("impl {}", struct_name)) || 
                           (line.contains("impl") && line.contains(struct_name)) {
                            in_impl = true;
                        }
                        
                        if in_impl {
                            impl_lines.push(line);
                            brace_count += line.matches('{').count();
                            brace_count -= line.matches('}').count();
                            
                            if brace_count == 0 && impl_lines.len() > 1 {
                                break;
                            }
                        }
                    }
                    
                    if !impl_lines.is_empty() {
                        return Ok(impl_lines.join("\n"));
                    }
                }
            }
        }
        
        Err(rmcp::ErrorData::resource_not_found(format!("Implementation block not found for: {}", struct_name), None))
    }
}

impl Default for CodeResources {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}