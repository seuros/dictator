//! Tool implementations with dynamic visibility based on config state

use async_trait::async_trait;
use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use super::config_exists;
use crate::mcp::handlers::{
    handle_dictator, handle_occupy, handle_stalint, handle_stalint_unwatch, handle_stalint_watch,
};
use crate::mcp::state::ServerState;

/// Stalint structural linting tool
pub struct StalintTool {
    pub state: Arc<Mutex<ServerState>>,
}

#[async_trait]
impl Tool for StalintTool {
    fn name(&self) -> &'static str {
        "stalint"
    }

    fn description(&self) -> Option<&str> {
        Some("Run structural linting checks on files (read-only analysis)")
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn execute(&self, ctx: ExecutionContext<'_>) -> Result<ToolOutput, ToolError> {
        let response = handle_stalint(Value::Null, Some(ctx.params), Arc::clone(&self.state));

        if let Some(error) = response.error {
            return Err(ToolError::Execution(error.message));
        }

        let result = response
            .result
            .ok_or_else(|| ToolError::Execution("No result from stalint handler".to_string()))?;

        let content = TextContent::new(serde_json::to_string_pretty(&result).unwrap_or_default());
        Ok(ToolOutput::Content(vec![Box::new(content)]))
    }
}

/// Dictator auto-fix tool
pub struct DictatorTool {
    pub state: Arc<Mutex<ServerState>>,
}

#[async_trait]
impl Tool for DictatorTool {
    fn name(&self) -> &'static str {
        "dictator"
    }

    fn description(&self) -> Option<&str> {
        Some("Auto-fix structural violations (requires write permissions)")
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn execute(&self, ctx: ExecutionContext<'_>) -> Result<ToolOutput, ToolError> {
        let can_write = self.state.lock().unwrap().can_write;
        if !can_write {
            return Err(ToolError::Execution(
                "Write operations disabled in read-only mode".to_string(),
            ));
        }

        let response = handle_dictator(Value::Null, Some(ctx.params), Arc::clone(&self.state));

        if let Some(error) = response.error {
            return Err(ToolError::Execution(error.message));
        }

        let result = response
            .result
            .ok_or_else(|| ToolError::Execution("No result from dictator handler".to_string()))?;

        let content = TextContent::new(serde_json::to_string_pretty(&result).unwrap_or_default());
        Ok(ToolOutput::Content(vec![Box::new(content)]))
    }
}

/// Watch files for structural changes
pub struct StalintWatchTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
}

#[async_trait]
impl Tool for StalintWatchTool {
    fn name(&self) -> &'static str {
        "stalint_watch"
    }

    fn description(&self) -> Option<&str> {
        Some("Start watching paths for structural violations")
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn execute(&self, ctx: ExecutionContext<'_>) -> Result<ToolOutput, ToolError> {
        ctx.logger.info("Starting path watch...");

        let (string_tx, mut string_rx) = tokio::sync::mpsc::channel::<String>(100);

        let notification_tx = self.notification_tx.clone();
        tokio::spawn(async move {
            while let Some(notif_str) = string_rx.recv().await {
                if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&notif_str)
                    && notification_tx.send(notif).is_err()
                {
                    break;
                }
            }
        });

        let response = handle_stalint_watch(
            Value::Null,
            Some(ctx.params),
            Arc::clone(&self.state),
            string_tx,
        );

        if let Some(error) = response.error {
            return Err(ToolError::Execution(error.message));
        }

        let result = response.result.ok_or_else(|| {
            ToolError::Execution("No result from stalint_watch handler".to_string())
        })?;

        // Swap tools: hide watch, show unwatch
        ctx.session.batch(|batch| {
            batch.hide_tool("stalint_watch");
            batch.add_tool(Arc::new(StalintUnwatchTool {
                state: Arc::clone(&self.state),
                notification_tx: self.notification_tx.clone(),
            }));
        });

        ctx.logger
            .info("Watch started. stalint_unwatch now available.");

        let content = TextContent::new(serde_json::to_string_pretty(&result).unwrap_or_default());
        Ok(ToolOutput::Content(vec![Box::new(content)]))
    }
}

/// Unwatch files
pub struct StalintUnwatchTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
}

#[async_trait]
impl Tool for StalintUnwatchTool {
    fn name(&self) -> &'static str {
        "stalint_unwatch"
    }

    fn description(&self) -> Option<&str> {
        Some("Stop watching paths")
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn execute(&self, ctx: ExecutionContext<'_>) -> Result<ToolOutput, ToolError> {
        ctx.logger.info("Stopping path watch...");

        let (string_tx, mut string_rx) = tokio::sync::mpsc::channel::<String>(100);

        let notification_tx = self.notification_tx.clone();
        tokio::spawn(async move {
            while let Some(notif_str) = string_rx.recv().await {
                if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&notif_str)
                    && notification_tx.send(notif).is_err()
                {
                    break;
                }
            }
        });

        let response = handle_stalint_unwatch(Value::Null, Arc::clone(&self.state), string_tx);

        if let Some(error) = response.error {
            return Err(ToolError::Execution(error.message));
        }

        let result = response.result.ok_or_else(|| {
            ToolError::Execution("No result from stalint_unwatch handler".to_string())
        })?;

        // Swap tools: hide unwatch, show watch
        ctx.session.batch(|batch| {
            batch.remove_tool("stalint_unwatch");
            batch.unhide_tool("stalint_watch");
        });

        ctx.logger
            .info("Watch stopped. stalint_watch now available.");

        let content = TextContent::new(serde_json::to_string_pretty(&result).unwrap_or_default());
        Ok(ToolOutput::Content(vec![Box::new(content)]))
    }
}

/// Initialize .dictate.toml
pub struct OccupyTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
}

#[async_trait]
impl Tool for OccupyTool {
    fn name(&self) -> &'static str {
        "occupy"
    }

    fn description(&self) -> Option<&str> {
        Some("Initialize .dictate.toml configuration file (requires write permissions)")
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        !config_exists()
    }

    async fn execute(&self, ctx: ExecutionContext<'_>) -> Result<ToolOutput, ToolError> {
        ctx.logger.info("Initializing .dictate.toml...");

        let can_write = self.state.lock().unwrap().can_write;
        if !can_write {
            return Err(ToolError::Execution(
                "Write operations disabled in read-only mode".to_string(),
            ));
        }

        let (string_tx, mut string_rx) = tokio::sync::mpsc::channel::<String>(100);

        let notification_tx = self.notification_tx.clone();
        tokio::spawn(async move {
            while let Some(notif_str) = string_rx.recv().await {
                if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&notif_str)
                    && notification_tx.send(notif).is_err()
                {
                    break;
                }
            }
        });

        let response = handle_occupy(Value::Null, Arc::clone(&self.state), string_tx);

        if let Some(error) = response.error {
            return Err(ToolError::Execution(error.message));
        }

        let result = response
            .result
            .ok_or_else(|| ToolError::Execution("No result from occupy handler".to_string()))?;

        let content = TextContent::new(serde_json::to_string_pretty(&result).unwrap_or_default());
        Ok(ToolOutput::Content(vec![Box::new(content)]))
    }
}
