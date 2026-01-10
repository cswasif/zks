//! Review prompts for ZKS MCP server
//! 
//! Provides prompt templates for code reviews, API design reviews,
//! and documentation reviews.

use rmcp::{Prompt, PromptArgument};

#[derive(Clone)]
pub struct ReviewPrompts;

impl ReviewPrompts {
    pub fn new() -> Self {
        Self
    }

    pub fn prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt {
                name: "zks_code_review".into(),
                description: Some("Code review checklist".into()),
                arguments: vec![
                    PromptArgument {
                        name: "pr_diff".into(),
                        description: Some("Pull request diff".into()),
                        required: true,
                    },
                ],
            },
        ]
    }
}

impl Default for ReviewPrompts {
    fn default() -> Self {
        Self::new()
    }
}