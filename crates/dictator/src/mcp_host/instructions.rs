//! Per-session server instructions

use std::sync::LazyLock;

use mcp_host::prelude::*;

use super::config_exists;

pub const BASE: &str = "Before any commit, call this server's `stalint` tool (an MCP tool, not a \
     shell command) to lint uncommitted changes. If it reports violations, call the `dictator` \
     tool to auto-fix them or fix them by hand. User expects disciplined agents.";

/// The territory has no `.dictate.toml`: lint tools stay hidden until `occupy` runs.
const UNOCCUPIED: &str = "No .dictate.toml in this workspace. Run the `occupy` tool to create \
     one, then customize it for the codebase. Lint and fix tools appear once it exists.";

// Client quirks, as mcp-host client specs: a `clientInfo.name`, optionally
// with version constraints (`claude-code < 2.93`) so a quirk ends at the
// release that fixes it.

/// Clients that never list MCP resources to their model: the instructions
/// inline the catalog instead.
const HIDES_RESOURCES: &[&str] = &["claude-code"];

/// Clients that replace any failed resource read with their own text: the
/// error comes back as content so the model learns why.
pub const HIDES_RESOURCE_ERRORS: &[&str] = &["claude-code"];

/// Instructions for the initializing client; `None` keeps [`BASE`].
pub fn for_client(ctx: &InstructionsContext<'_>) -> Option<String> {
    let catalog = if model_sees_resources(ctx.client) {
        None
    } else {
        ctx.resource_catalog()
    };
    compose(config_exists(), catalog)
}

fn compose(occupied: bool, catalog: Option<String>) -> Option<String> {
    if occupied && catalog.is_none() {
        return None;
    }

    let mut text = String::from(if occupied { BASE } else { UNOCCUPIED });
    if let Some(catalog) = catalog {
        text.push_str("\n\nResources (ReadMcpResourceTool):\n");
        text.push_str(&catalog);
    }
    Some(text)
}

fn model_sees_resources(client: &Implementation) -> bool {
    static HIDDEN: LazyLock<Vec<ClientMatcher>> = LazyLock::new(|| {
        HIDES_RESOURCES
            .iter()
            .map(|spec| spec.parse().expect("valid client spec"))
            .collect()
    });
    !HIDDEN.iter().any(|matcher| matcher.matches(client))
}

#[cfg(test)]
mod tests;
