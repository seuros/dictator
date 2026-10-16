use super::*;
use mcp_host::server::multiplexer::JsonRpcClientRequest;
use mcp_host::transport::traits::{IncomingMessage, Transport};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

async fn call(
    client: &mut InProcessTransport,
    method: &'static str,
    params: Value,
) -> (Value, Vec<Value>) {
    client
        .send_request(JsonRpcClientRequest::new(
            NEXT_REQUEST.fetch_add(1, Ordering::Relaxed).to_string(),
            method,
            Some(params),
        ))
        .await
        .unwrap();
    let mut notifications = Vec::new();
    loop {
        match tokio::time::timeout(Duration::from_secs(10), client.read_incoming())
            .await
            .expect("server response timed out")
            .unwrap()
        {
            IncomingMessage::Request(notification) => {
                assert!(notification.id.is_none(), "{notification:?}");
                notifications.push(serde_json::to_value(notification).unwrap());
            }
            IncomingMessage::Response(response) => {
                assert!(response.error.is_none(), "{response:?}");
                return (response.result.unwrap(), notifications);
            }
        }
    }
}

#[tokio::test]
async fn lint_progress_uses_only_the_callers_token_and_resources_are_subscribable() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join(".dictate.toml"), "[decree.supreme]\n").unwrap();
    std::fs::write(directory.path().join("probe.txt"), "clean\n").unwrap();

    for version in ["2025-06-18", "2025-11-25"] {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let server = Arc::new(
            Server::builder("dictator-test", "1")
                .with_tools(true)
                .with_resources(true, true)
                .with_logging()
                .build(),
        );
        // MOCKED: omit only the process-CWD config visibility guard so this
        // fixture exercises the real handler against an isolated workspace.
        // Never change process CWD or create a shared .dictate.toml in tests.
        let tools = McpToolRouter::new().with_tool(
            DictatorTools::stalint_tool_info(),
            DictatorTools::stalint_handler,
            None,
        );
        server
            .register_router(
                McpRouter::new(
                    tools,
                    McpPromptRouter::new(),
                    McpResourceRouter::new(),
                    McpResourceTemplateRouter::new(),
                ),
                Arc::new(DictatorTools {
                    state: Arc::clone(&state),
                }),
            )
            .unwrap();
        server
            .register_router(
                DictatorResources::router().unwrap(),
                Arc::new(DictatorResources {
                    state: Arc::clone(&state),
                }),
            )
            .unwrap();
        let watch = StalintWatchTool {
            state: Arc::clone(&state),
        };
        assert_eq!(watch.input_schema()["required"], json!(["paths"]));
        let (transport, mut client) = InProcessTransport::pair();
        let running = Arc::clone(&server);
        let task = tokio::spawn(async move { running.run(transport).await.unwrap() });
        let (result, _) = call(
            &mut client,
            "initialize",
            json!({
                "protocolVersion": version,
                "capabilities": {},
                "clientInfo": {"name": "dictator-test", "version": "1"}
            }),
        )
        .await;
        assert_eq!(result["protocolVersion"], version);
        client
            .send_notification(JsonRpcNotification::new("notifications/initialized", None))
            .await
            .unwrap();

        let (tools, _) = call(&mut client, "tools/list", json!({})).await;
        assert!(
            tools["tools"]
                .as_array()
                .unwrap()
                .iter()
                .any(|tool| tool["name"] == "stalint")
        );
        for token in [None, Some("owned-token")] {
            let mut params = json!({
                "name": "stalint",
                "arguments": {"workspace": directory.path().to_str().unwrap()}
            });
            if let Some(token) = token {
                params["_meta"] = json!({"progressToken": token});
            }
            let (result, notifications) = call(&mut client, "tools/call", params).await;
            assert_ne!(result["isError"], true, "{result}");
            let progress = notifications
                .iter()
                .filter(|n| n["method"] == "notifications/progress")
                .collect::<Vec<_>>();
            if token.is_none() {
                assert!(progress.is_empty(), "{progress:?}");
            } else {
                assert!(!progress.is_empty());
                let mut last = -1.0;
                for notification in progress {
                    assert_eq!(notification["params"]["progressToken"], "owned-token");
                    let value = notification["params"]["progress"].as_f64().unwrap();
                    assert!(value > last);
                    last = value;
                }
            }
        }
        let (mood, _) = call(
            &mut client,
            "resources/read",
            json!({"uri": super::super::resources::MOOD_URI}),
        )
        .await;
        assert_eq!(mood["contents"][0]["mimeType"], "application/json");
        call(
            &mut client,
            "resources/subscribe",
            json!({"uri": super::super::resources::MOOD_URI}),
        )
        .await;
        state.lock().unwrap().record_violations(5);
        server.notify_resource_updated(super::super::resources::MOOD_URI);
        let update = tokio::time::timeout(Duration::from_secs(10), client.read_incoming())
            .await
            .unwrap()
            .unwrap();
        let IncomingMessage::Request(update) = update else {
            panic!("expected resource notification")
        };
        assert_eq!(update.method, "notifications/resources/updated");
        assert_eq!(
            update.params.unwrap()["uri"],
            super::super::resources::MOOD_URI
        );
        client.shutdown().await.unwrap();
        task.abort();
    }
}
