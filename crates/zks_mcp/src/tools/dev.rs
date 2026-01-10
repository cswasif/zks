//! Development tools for ZKS MCP server
//! 
//! Provides tools for ZKS development operations including building, testing,
//! formatting, linting, documentation generation, and benchmarking.

use rmcp::{tool, tool_router, model::*, ErrorData as McpError};
use rmcp::handler::server::wrapper::Parameters;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::process::Command;

#[derive(Clone)]
pub struct DevTools;

impl DevTools {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DevTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BuildParams {
    pub crate_name: Option<String>,
    pub target: Option<String>,
    pub features: Option<String>,
    pub release: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestParams {
    pub crate_name: Option<String>,
    pub test_filter: Option<String>,
    pub test_type: Option<String>, // "unit", "integration", "doc"
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FmtParams {
    pub path: Option<String>,
    pub check_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClippyParams {
    pub crate_name: Option<String>,
    pub allow_warnings: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocParams {
    pub crate_name: Option<String>,
    pub open: Option<bool>,
    pub no_deps: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BenchParams {
    pub bench_name: Option<String>,
    pub crate_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateBindingsParams {
    pub target: String, // "wasm" or "uniffi"
    pub crate_name: Option<String>,
}

#[tool_router]
impl DevTools {
    #[tool(description = "Build ZKS crates with cargo")]
    async fn zks_build(
        &self,
        params: Parameters<BuildParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        
        if params.release.unwrap_or(false) {
            cmd.arg("--release");
        }
        
        if let Some(crate_name) = &params.crate_name {
            cmd.arg("--package").arg(crate_name);
        }
        
        if let Some(target) = &params.target {
            cmd.arg("--target").arg(target);
        }
        
        if let Some(features) = &params.features {
            cmd.arg("--features").arg(features);
        }
        
        let output = cmd.output()
            .map_err(|e| McpError::internal_error(format!("Failed to execute cargo build: {}", e), None))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();
        
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "success": success,
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr,
            "command": format!("{:?}", cmd)
        }).to_string())]))
    }

    #[tool(description = "Run tests with cargo")]
    async fn zks_test(
        &self,
        params: Parameters<TestParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let mut cmd = Command::new("cargo");
        
        match params.test_type.as_deref() {
            Some("doc") => cmd.arg("test").arg("--doc"),
            _ => cmd.arg("test"),
        };
        
        if let Some(crate_name) = &params.crate_name {
            cmd.arg("--package").arg(crate_name);
        }
        
        if let Some(filter) = &params.test_filter {
            cmd.arg(filter);
        }
        
        let output = cmd.output()
            .map_err(|e| McpError::internal_error(format!("Failed to execute cargo test: {}", e), None))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();
        
        // Parse test results from output
        let output_str = stdout.to_string();
        let passed = output_str.lines()
            .find(|line| line.contains("test result:"))
            .and_then(|line| {
                line.split_whitespace()
                    .find(|word| word.parse::<u32>().is_ok())
                    .and_then(|num| num.parse::<u32>().ok())
            })
            .unwrap_or(0);
        
        let failed = output_str.lines()
            .find(|line| line.contains("failed"))
            .and_then(|line| {
                line.split_whitespace()
                    .find(|word| word.parse::<u32>().is_ok())
                    .and_then(|num| num.parse::<u32>().ok())
            })
            .unwrap_or(0);
        
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "success": success,
            "passed": passed,
            "failed": failed,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code()
        }).to_string())]))
    }

    #[tool(description = "Format code with rustfmt")]
    async fn zks_fmt(
        &self,
        params: Parameters<FmtParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let mut cmd = Command::new("cargo");
        cmd.arg("fmt");
        
        if params.check_only.unwrap_or(false) {
            cmd.arg("--check");
        }
        
        if let Some(path) = &params.path {
            cmd.arg("--manifest-path").arg(format!("{}/Cargo.toml", path));
        }
        
        let output = cmd.output()
            .map_err(|e| McpError::internal_error(format!("Failed to execute cargo fmt: {}", e), None))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();
        
        let formatted_files = if success && !params.check_only.unwrap_or(false) {
            // Count files that were formatted
            stdout.lines().count() as u32
        } else {
            0
        };
        
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "success": success,
            "formatted_files": formatted_files,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code()
        }).to_string())]))
    }

    #[tool(description = "Run clippy lints")]
    async fn zks_clippy(
        &self,
        params: Parameters<ClippyParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let mut cmd = Command::new("cargo");
        cmd.arg("clippy");
        
        if let Some(crate_name) = &params.crate_name {
            cmd.arg("--package").arg(crate_name);
        }
        
        if !params.allow_warnings.unwrap_or(true) {
            cmd.arg("--").arg("-D").arg("warnings");
        }
        
        let output = cmd.output()
            .map_err(|e| McpError::internal_error(format!("Failed to execute cargo clippy: {}", e), None))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();
        
        // Count warnings and errors
        let output_str = stdout.to_string() + &stderr.to_string();
        let warnings = output_str.matches("warning:").count() as u32;
        let errors = output_str.matches("error:").count() as u32;
        
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "success": success,
            "warnings": warnings,
            "errors": errors,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code()
        }).to_string())]))
    }

    #[tool(description = "Generate documentation with cargo doc")]
    async fn zks_doc(
        &self,
        params: Parameters<DocParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let mut cmd = Command::new("cargo");
        cmd.arg("doc");
        
        if let Some(crate_name) = &params.crate_name {
            cmd.arg("--package").arg(crate_name);
        }
        
        if params.no_deps.unwrap_or(false) {
            cmd.arg("--no-deps");
        }
        
        let output = cmd.output()
            .map_err(|e| McpError::internal_error(format!("Failed to execute cargo doc: {}", e), None))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();
        
        // Determine documentation path
        let doc_path = if let Some(crate_name) = &params.crate_name {
            format!("target/doc/{}/index.html", crate_name.replace('-', "_"))
        } else {
            "target/doc/index.html".to_string()
        };
        
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "success": success,
            "doc_path": doc_path,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code(),
            "open_command": if params.open.unwrap_or(false) { 
                format!("Open: {}", doc_path) 
            } else { 
                "".to_string() 
            }
        }).to_string())]))
    }

    #[tool(description = "Run benchmarks")]
    async fn zks_bench(
        &self,
        params: Parameters<BenchParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let mut cmd = Command::new("cargo");
        cmd.arg("bench");
        
        if let Some(crate_name) = &params.crate_name {
            cmd.arg("--package").arg(crate_name);
        }
        
        if let Some(bench_name) = &params.bench_name {
            cmd.arg(bench_name);
        }
        
        let output = cmd.output()
            .map_err(|e| McpError::internal_error(format!("Failed to execute cargo bench: {}", e), None))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();
        
        // Parse benchmark results
        let output_str = stdout.to_string();
        let mut results = Vec::new();
        
        for line in output_str.lines() {
            if line.contains("bench:") && line.contains("ns/iter") {
                if let Some(bench_name) = line.split_whitespace().next() {
                    if let Some(ns_per_iter) = line.split("ns/iter").next()
                        .and_then(|s| s.split_whitespace().last())
                        .and_then(|s| s.parse::<u64>().ok()) {
                        results.push(serde_json::json!({
                            "name": bench_name,
                            "ns_per_iter": ns_per_iter,
                            "unit": "ns/iter"
                        }));
                    }
                }
            }
        }
        
        Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
            "success": success,
            "results": results,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": output.status.code()
        }).to_string())]))
    }

    #[tool(description = "Generate FFI bindings (WASM or UniFFI)")]
    async fn zks_generate_bindings(
        &self,
        params: Parameters<GenerateBindingsParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        
        match params.target.as_str() {
            "wasm" => {
                let mut cmd = Command::new("wasm-pack");
                cmd.arg("build");
                cmd.arg("--target").arg("web");
                
                if let Some(crate_name) = &params.crate_name {
                    cmd.arg("--").arg("--package").arg(crate_name);
                }
                
                let output = cmd.output()
                    .map_err(|e| McpError::internal_error(format!("Failed to execute wasm-pack: {}", e), None))?;
                
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let success = output.status.success();
                
                Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
                    "success": success,
                    "target": "wasm",
                    "output_path": "pkg/",
                    "stdout": stdout,
                    "stderr": stderr,
                    "exit_code": output.status.code()
                }).to_string())]))
            }
            "uniffi" => {
                let mut cmd = Command::new("cargo");
                cmd.arg("uniffi-bindgen");
                cmd.arg("generate");
                
                if let Some(crate_name) = &params.crate_name {
                    cmd.arg("--library").arg(format!("target/debug/lib{}.so", crate_name.replace('-', "_")));
                }
                
                let output = cmd.output()
                    .map_err(|e| McpError::internal_error(format!("Failed to execute cargo uniffi-bindgen: {}", e), None))?;
                
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let success = output.status.success();
                
                Ok(CallToolResult::success(vec![Content::text(serde_json::json!({
                    "success": success,
                    "target": "uniffi",
                    "output_path": "bindings/",
                    "stdout": stdout,
                    "stderr": stderr,
                    "exit_code": output.status.code()
                }).to_string())]))
            }
            _ => Err(McpError::invalid_params("Invalid target. Use 'wasm' or 'uniffi'".to_string(), None))
        }
    }
}