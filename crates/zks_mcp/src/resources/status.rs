//! Status resources for ZKS MCP server
//! 
//! Provides access to build status, test results, code coverage,
//! dependency audits, and version information.

use rmcp::model::{ResourceTemplate, ResourceContents, RawResourceTemplate};
use rmcp::ErrorData;
use std::process::Command;
use std::collections::HashMap;
use serde_json::json;

#[derive(Clone)]
pub struct StatusResource {
    zks_protocol_root: String,
}

impl StatusResource {
    pub fn new(zks_protocol_root: String) -> Self {
        Self { zks_protocol_root }
    }

    pub fn resources(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://status/build".into(),
                    name: "Build Status".into(),
                    title: None,
                    description: Some("Current build status".into()),
                    mime_type: Some("application/json".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://status/tests".into(),
                    name: "Test Results".into(),
                    title: None,
                    description: Some("Latest test results".into()),
                    mime_type: Some("application/json".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://status/coverage".into(),
                    name: "Code Coverage".into(),
                    title: None,
                    description: Some("Code coverage metrics".into()),
                    mime_type: Some("application/json".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://status/deps".into(),
                    name: "Dependency Audit".into(),
                    title: None,
                    description: Some("Dependency security audit".into()),
                    mime_type: Some("application/json".into()),
                },
                annotations: None,
            },
            ResourceTemplate {
                raw: RawResourceTemplate {
                    uri_template: "zks://status/versions".into(),
                    name: "Crate Versions".into(),
                    title: None,
                    description: Some("Version information for all crates".into()),
                    mime_type: Some("application/json".into()),
                },
                annotations: None,
            },
        ]
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContents, ErrorData> {
        match uri {
            "zks://status/build" => {
                let build_status = self.get_build_status().await?;
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: json!(build_status).to_string(),
                    meta: None,
                })
            }
            "zks://status/tests" => {
                let test_results = self.get_test_results().await?;
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: json!(test_results).to_string(),
                    meta: None,
                })
            }
            "zks://status/coverage" => {
                let coverage = self.get_coverage().await?;
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: json!(coverage).to_string(),
                    meta: None,
                })
            }
            "zks://status/deps" => {
                let deps = self.get_dependency_audit().await?;
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: json!(deps).to_string(),
                    meta: None,
                })
            }
            "zks://status/versions" => {
                let versions = self.get_versions().await?;
                Ok(ResourceContents::TextResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: json!(versions).to_string(),
                    meta: None,
                })
            }
            _ => Err(rmcp::ErrorData::resource_not_found(format!("Unknown status resource: {}", uri), None))
        }
    }

    async fn get_build_status(&self) -> Result<HashMap<String, serde_json::Value>, rmcp::ErrorData> {
        let mut status = HashMap::new();
        
        // Check if workspace builds successfully
        let output = Command::new("cargo")
            .args(&["check", "--workspace"])
            .current_dir(&self.zks_protocol_root)
            .output()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to run cargo check: {}", e), None))?;

        status.insert("workspace_build".to_string(), json!(output.status.success()));
        status.insert("build_output".to_string(), json!(String::from_utf8_lossy(&output.stderr).to_string()));

        // Check individual crate builds
        let crates = vec!["zks_sdk", "zks_crypt", "zks_pqcrypto", "zks_proto", "zks_wire", "zks_types", "zks_mcp"];
        let mut crate_status = HashMap::new();
        
        for crate_name in crates {
            let crate_output = Command::new("cargo")
                .args(&["check", "-p", crate_name])
                .current_dir(&self.zks_protocol_root)
                .output()
                .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to check {}: {}", crate_name, e), None))?;
            
            crate_status.insert(crate_name.to_string(), json!(crate_output.status.success()));
        }
        
        status.insert("crate_builds".to_string(), json!(crate_status));
        Ok(status)
    }

    async fn get_test_results(&self) -> Result<HashMap<String, serde_json::Value>, rmcp::ErrorData> {
        let mut results = HashMap::new();
        
        // Run tests and capture results
        let output = Command::new("cargo")
            .args(&["test", "--workspace", "--", "--nocapture"])
            .current_dir(&self.zks_protocol_root)
            .output()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to run cargo test: {}", e), None))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let error_str = String::from_utf8_lossy(&output.stderr);
        
        // Parse test results
        let passed = output_str.matches("test result:").count() > 0 && output_str.contains("passed");
        let failed = output_str.contains("FAILED") || !output.status.success();
        
        results.insert("passed".to_string(), json!(passed));
        results.insert("failed".to_string(), json!(failed));
        results.insert("output".to_string(), json!(output_str.to_string()));
        results.insert("errors".to_string(), json!(error_str.to_string()));
        results.insert("exit_code".to_string(), json!(output.status.code()));

        Ok(results)
    }

    async fn get_coverage(&self) -> Result<HashMap<String, serde_json::Value>, rmcp::ErrorData> {
        let mut coverage = HashMap::new();
        
        // Try to run cargo tarpaulin if available
        let output = Command::new("cargo")
            .args(&["tarpaulin", "--out", "Json"])
            .current_dir(&self.zks_protocol_root)
            .output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Parse coverage output (simplified)
                let line_coverage = if output_str.contains("Coverage:") {
                    let coverage_line = output_str.lines()
                        .find(|line| line.contains("Coverage:"))
                        .unwrap_or("Coverage: 0%");
                    
                    // Extract percentage
                    coverage_line.split('%').next()
                        .and_then(|s| s.split_whitespace().last())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                coverage.insert("line_coverage".to_string(), json!(line_coverage));
                coverage.insert("branch_coverage".to_string(), json!(0.0)); // Placeholder
                coverage.insert("function_coverage".to_string(), json!(0.0)); // Placeholder
                coverage.insert("tool".to_string(), json!("cargo-tarpaulin"));
            }
            Err(_) => {
                // Fallback: no coverage tool available
                coverage.insert("line_coverage".to_string(), json!(0.0));
                coverage.insert("branch_coverage".to_string(), json!(0.0));
                coverage.insert("function_coverage".to_string(), json!(0.0));
                coverage.insert("tool".to_string(), json!("none"));
                coverage.insert("message".to_string(), json!("cargo-tarpaulin not available"));
            }
        }

        Ok(coverage)
    }

    async fn get_dependency_audit(&self) -> Result<HashMap<String, serde_json::Value>, rmcp::ErrorData> {
        let mut audit = HashMap::new();
        
        // Try to run cargo audit
        let output = Command::new("cargo")
            .args(&["audit", "--json"])
            .current_dir(&self.zks_protocol_root)
            .output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Parse audit output (simplified)
                let vulnerabilities = if output_str.contains("vulnerabilities") {
                    output_str.matches("vulnerabilities").count() as u32
                } else {
                    0
                };

                let warnings = if output_str.contains("warning") {
                    output_str.matches("warning").count() as u32
                } else {
                    0
                };

                audit.insert("vulnerabilities".to_string(), json!(vulnerabilities));
                audit.insert("warnings".to_string(), json!(warnings));
                audit.insert("tool".to_string(), json!("cargo-audit"));
                audit.insert("output".to_string(), json!(output_str.to_string()));
            }
            Err(_) => {
                // Fallback: no audit tool available
                audit.insert("vulnerabilities".to_string(), json!(0));
                audit.insert("warnings".to_string(), json!(0));
                audit.insert("tool".to_string(), json!("none"));
                audit.insert("message".to_string(), json!("cargo-audit not available"));
            }
        }

        Ok(audit)
    }

    async fn get_versions(&self) -> Result<HashMap<String, serde_json::Value>, rmcp::ErrorData> {
        let mut versions = HashMap::new();
        
        // Get workspace version from Cargo.toml
        let workspace_cargo = std::fs::read_to_string(format!("{}/Cargo.toml", self.zks_protocol_root))
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to read workspace Cargo.toml: {}", e), None))?;
        
        let workspace_toml: toml::Value = toml::from_str(&workspace_cargo)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to parse workspace Cargo.toml: {}", e), None))?;
        
        if let Some(workspace_version) = workspace_toml.get("workspace").and_then(|w| w.get("package")).and_then(|p| p.get("version")).and_then(|v| v.as_str()) {
            versions.insert("workspace".to_string(), json!(workspace_version));
        }

        // Get individual crate versions
        let crates = vec!["zks_sdk", "zks_crypt", "zks_pqcrypto", "zks_proto", "zks_wire", "zks_types", "zks_mcp"];
        let mut crate_versions = HashMap::new();
        
        for crate_name in crates {
            let crate_path = format!("{}/crates/{}/Cargo.toml", self.zks_protocol_root, crate_name);
            if let Ok(cargo_content) = std::fs::read_to_string(&crate_path) {
                if let Ok(cargo_toml) = toml::from_str::<toml::Value>(&cargo_content) {
                    if let Some(version) = cargo_toml.get("package").and_then(|p| p.get("version")).and_then(|v| v.as_str()) {
                        crate_versions.insert(crate_name.to_string(), json!(version));
                    }
                }
            }
        }
        
        versions.insert("crates".to_string(), json!(crate_versions));
        
        // Get Rust version
        let rust_output = Command::new("rustc")
            .args(&["--version"])
            .output()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to get Rust version: {}", e), None))?;
        
        let rust_version = String::from_utf8_lossy(&rust_output.stdout).trim().to_string();
        versions.insert("rust".to_string(), json!(rust_version));

        Ok(versions)
    }
}

impl Default for StatusResource {
    fn default() -> Self {
        Self::new(".".to_string())
    }
}