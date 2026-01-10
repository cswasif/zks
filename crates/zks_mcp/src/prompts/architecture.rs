//! Architecture prompts for ZKS MCP server
//! 
//! Provides prompt templates for architecture overviews, crate guides,
//! and coding pattern explanations.

use rmcp::{Prompt, PromptArgument};

#[derive(Clone)]
pub struct ArchitecturePrompts;

impl ArchitecturePrompts {
    pub fn new() -> Self {
        Self
    }

    pub fn prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt {
                name: "zks_architecture_overview".into(),
                description: Some("Explain ZKS architecture".into()),
                arguments: vec![
                    PromptArgument {
                        name: "detail_level".into(),
                        description: Some("Level of detail: 'high' | 'medium' | 'low'".into()),
                        required: false,
                    },
                ],
            },
        ]
    }
}

impl Default for ArchitecturePrompts {
    fn default() -> Self {
        Self::new()
    }
}