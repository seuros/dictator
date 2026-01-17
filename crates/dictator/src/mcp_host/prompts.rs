//! Prompt implementations with dynamic visibility based on config state

use async_trait::async_trait;
use mcp_host::prelude::*;
use std::sync::{Arc, Mutex};

use super::config_exists;
use crate::mcp::state::ServerState;

const DEFAULT_CONFIG: &str = include_str!("../../templates/default.dictate.toml");

/// Onboarding prompt - shown when .dictate.toml doesn't exist
pub struct OnboardPrompt {
    #[allow(dead_code)]
    state: Arc<Mutex<ServerState>>,
}

impl OnboardPrompt {
    pub const fn new(state: Arc<Mutex<ServerState>>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Prompt for OnboardPrompt {
    fn name(&self) -> &'static str {
        "onboard"
    }

    fn description(&self) -> Option<&str> {
        Some("Introduction to Dictator for new projects")
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        !config_exists()
    }

    async fn get(&self, _ctx: ExecutionContext<'_>) -> Result<GetPromptResult, PromptError> {
        let cwd = std::env::current_dir()
            .map_err(|e| PromptError::Execution(format!("Failed to get cwd: {e}")))?;
        let config_path = cwd.join(".dictate.toml");

        // Create the config file
        if !config_path.exists() {
            std::fs::write(&config_path, DEFAULT_CONFIG)
                .map_err(|e| PromptError::Execution(format!("Failed to create config: {e}")))?;
        }

        let message = format!(
            "I've created `.dictate.toml` in this project.\n\n\
            Please read the file and customize it for this codebase:\n\
            - Enable/disable specific decrees\n\
            - Configure paths to include/exclude\n\
            - Set language-specific rules\n\n\
            The config file is at: {}",
            config_path.display()
        );

        Ok(GetPromptResult {
            description: Some("Dictator configuration created".to_string()),
            messages: vec![PromptMessage::user(message)],
        })
    }
}

/// Pre-commit check prompt - shown when .dictate.toml exists
pub struct PreCommitPrompt {
    #[allow(dead_code)]
    state: Arc<Mutex<ServerState>>,
}

impl PreCommitPrompt {
    pub const fn new(state: Arc<Mutex<ServerState>>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Prompt for PreCommitPrompt {
    fn name(&self) -> &'static str {
        "pre_commit"
    }

    fn description(&self) -> Option<&str> {
        Some("Check staged files before commit")
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn get(&self, _ctx: ExecutionContext<'_>) -> Result<GetPromptResult, PromptError> {
        let staged = std::process::Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let files: Vec<&str> = staged.lines().filter(|l| !l.is_empty()).collect();

        if files.is_empty() {
            return Ok(GetPromptResult {
                description: Some("No staged files".to_string()),
                messages: vec![PromptMessage::user(
                    "No files staged for commit. Stage files with `git add` first.",
                )],
            });
        }

        Ok(GetPromptResult {
            description: Some(format!("Pre-commit check for {} files", files.len())),
            messages: vec![PromptMessage::user(format!(
                "Before committing, run `stalint` on these staged files:\n\n{}\n\n\
                If violations found, run `dictator` to auto-fix or address manually.",
                files.join("\n")
            ))],
        })
    }
}

/// Explain a specific violation type - shown when .dictate.toml exists
pub struct ExplainViolationPrompt {
    #[allow(dead_code)]
    state: Arc<Mutex<ServerState>>,
}

impl ExplainViolationPrompt {
    pub const fn new(state: Arc<Mutex<ServerState>>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Prompt for ExplainViolationPrompt {
    fn name(&self) -> &'static str {
        "explain_violation"
    }

    fn description(&self) -> Option<&str> {
        Some("Explain a specific violation type and how to fix it")
    }

    fn arguments(&self) -> Option<Vec<PromptArgument>> {
        Some(vec![PromptArgument {
            name: "violation_type".to_string(),
            description: Some("Type of violation to explain".to_string()),
            required: Some(true),
        }])
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn get(&self, ctx: ExecutionContext<'_>) -> Result<GetPromptResult, PromptError> {
        let violation_type = ctx
            .params
            .get("violation_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PromptError::InvalidArguments("Missing violation_type".to_string()))?;

        let explanation = match violation_type {
            "import_order" => {
                "Imports must be alphabetically sorted within groups. \
                Groups: std -> external crates -> internal modules. \
                Run `dictator` to auto-fix."
            }
            "naming_convention" => {
                "Files must follow snake_case for Rust/Python, \
                kebab-case for YAML/TOML, PascalCase for components. \
                Rename files manually."
            }
            "mod_structure" => {
                "Avoid premature mod.rs hierarchies. \
                Single-file modules preferred until complexity warrants splitting. \
                YAGNI compliance required."
            }
            _ => {
                "Unknown violation type. Check .dictate.toml for enabled decrees \
                and their documentation."
            }
        };

        Ok(GetPromptResult {
            description: Some(format!("Explanation: {violation_type}")),
            messages: vec![PromptMessage::user(explanation)],
        })
    }
}
