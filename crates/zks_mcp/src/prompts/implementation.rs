//! Implementation prompts for ZKS MCP server
//! 
//! Provides prompt templates for feature implementation guidance,
//! algorithm addition, protocol extension, and optimization.

use rmcp::{Prompt, PromptArgument};

#[derive(Clone)]
pub struct ImplementationPrompts;

impl ImplementationPrompts {
    pub fn new() -> Self {
        Self
    }

    pub fn prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt {
                name: "zks_implement_feature".into(),
                description: Some("Feature implementation guide".into()),
                arguments: vec![
                    PromptArgument {
                        name: "feature_name".into(),
                        description: Some("Name of the feature to implement".into()),
                        required: true,
                    },
                    PromptArgument {
                        name: "crate".into(),
                        description: Some("Target crate for implementation".into()),
                        required: false,
                    },
                ],
            },
        ]
    }
}

impl Default for ImplementationPrompts {
    fn default() -> Self {
        Self::new()
    }
}