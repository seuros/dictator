//! Prompt implementations using mcp-host macros

use mcp_host::prelude::*;
use serde_json::Value;

use super::config_exists;

const DEFAULT_CONFIG: &str = include_str!("../../templates/default.dictate.toml");

/// Dictator prompt handlers
pub struct DictatorPrompts;

#[mcp_router]
impl DictatorPrompts {
    /// Introduction to Dictator for new projects
    #[mcp_prompt(
        name = "onboard",
        visible = "!config_exists()"
    )]
    async fn onboard(&self, _ctx: Ctx<'_>, _args: Value) -> PromptResult {
        let cwd = std::env::current_dir()
            .map_err(|e| PromptError::Execution(format!("Failed to get cwd: {e}")))?;
        let config_path = cwd.join(".dictate.toml");

        if !config_path.exists() {
            std::fs::write(&config_path, DEFAULT_CONFIG)
                .map_err(|e| PromptError::Execution(format!("Failed to create config: {e}")))?;
        }

        prompt_with_description(
            "Dictator configuration created",
            vec![user_message(format!(
                "I've created `.dictate.toml` in this project.\n\n\
                Please read the file and customize it for this codebase:\n\
                - Enable/disable specific decrees\n\
                - Configure paths to include/exclude\n\
                - Set language-specific rules\n\n\
                The config file is at: {}",
                config_path.display()
            ))],
        )
    }

    /// Check staged files before commit
    #[mcp_prompt(
        name = "pre_commit",
        visible = "config_exists()"
    )]
    async fn pre_commit(&self, _ctx: Ctx<'_>, _args: Value) -> PromptResult {
        let staged = std::process::Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let files: Vec<&str> = staged.lines().filter(|l| !l.is_empty()).collect();

        if files.is_empty() {
            return prompt_with_description(
                "No staged files",
                vec![user_message(
                    "No files staged for commit. Stage files with `git add` first.",
                )],
            );
        }

        prompt_with_description(
            format!("Pre-commit check for {} files", files.len()),
            vec![user_message(format!(
                "Before committing, run `stalint` on these staged files:\n\n{}\n\n\
                If violations found, run `dictator` to auto-fix or address manually.",
                files.join("\n")
            ))],
        )
    }

    /// Explain a specific violation type and how to fix it
    #[mcp_prompt(
        name = "explain_violation",
        argument(
            name = "violation_type",
            description = "Type of violation to explain",
            required = true
        ),
        visible = "config_exists()"
    )]
    async fn explain_violation(&self, _ctx: Ctx<'_>, args: Value) -> PromptResult {
        let violation_type = args
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

        prompt_with_description(
            format!("Explanation: {violation_type}"),
            vec![user_message(explanation)],
        )
    }
}
