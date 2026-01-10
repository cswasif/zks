//! Security prompts for ZKS MCP server
//! 
//! Provides prompt templates for security reviews, cryptographic audits,
//! threat modeling, and penetration testing guidance.

use rmcp::{Prompt, PromptArgument};

#[derive(Clone)]
pub struct SecurityPrompts;

impl SecurityPrompts {
    pub fn new() -> Self {
        Self
    }

    pub fn prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt {
                name: "zks_security_review".into(),
                description: Some("Comprehensive security review for ZKS code".into()),
                arguments: vec![
                    PromptArgument {
                        name: "file_path".into(),
                        description: Some("Path to file to review".into()),
                        required: true,
                    },
                    PromptArgument {
                        name: "scope".into(),
                        description: Some("Review scope: 'full' | 'crypto' | 'network' | 'api'".into()),
                        required: false,
                    },
                ],
            },
            Prompt {
                name: "zks_crypto_audit".into(),
                description: Some("Audit cryptographic implementation against best practices".into()),
                arguments: vec![
                    PromptArgument {
                        name: "algorithm".into(),
                        description: Some("Algorithm being audited".into()),
                        required: true,
                    },
                ],
            },
        ]
    }
}

impl Default for SecurityPrompts {
    fn default() -> Self {
        Self::new()
    }
}