//! Watch lifecycle over real stdio, confined to a disposable workspace.

use serde_json::{Value, json};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};

async fn request(
    input: &mut ChildStdin,
    output: &mut BufReader<ChildStdout>,
    id: u64,
    method: &str,
    params: Value,
) -> Value {
    let line = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
    input
        .write_all(format!("{line}\n").as_bytes())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let mut line = String::new();
            assert!(
                output.read_line(&mut line).await.unwrap() > 0,
                "server exited"
            );
            let mut response: Value = serde_json::from_str(&line).unwrap();
            if response.get("id") == Some(&json!(id)) {
                assert!(response.get("error").is_none(), "{response}");
                return response["result"].take();
            }
        }
    })
    .await
    .expect("MCP response timed out")
}

fn tool_names(result: &Value) -> Vec<&str> {
    result["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn watch_and_unwatch_swap_the_session_catalog() {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(workspace.path().join(".dictate.toml"), "[decree.supreme]\n").unwrap();
    std::fs::write(workspace.path().join("probe.txt"), "clean\n").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_dictator"))
        .arg("mcp")
        .current_dir(workspace.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    request(
        &mut input,
        &mut output,
        1,
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "watch-regression", "version": "1"}
        }),
    )
    .await;
    input
        .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
        .await
        .unwrap();

    let tools = request(&mut input, &mut output, 2, "tools/list", json!({})).await;
    assert!(tool_names(&tools).contains(&"stalint_watch"));
    assert!(!tool_names(&tools).contains(&"stalint_unwatch"));
    let watched = request(
        &mut input,
        &mut output,
        3,
        "tools/call",
        json!({"name": "stalint_watch", "arguments": {"paths": ["probe.txt"]}}),
    )
    .await;
    assert_ne!(watched["isError"], true, "{watched}");
    let tools = request(&mut input, &mut output, 4, "tools/list", json!({})).await;
    assert!(!tool_names(&tools).contains(&"stalint_watch"));
    assert!(tool_names(&tools).contains(&"stalint_unwatch"));
    let stopped = request(
        &mut input,
        &mut output,
        5,
        "tools/call",
        json!({"name": "stalint_unwatch", "arguments": {}}),
    )
    .await;
    assert_ne!(stopped["isError"], true, "{stopped}");
    let tools = request(&mut input, &mut output, 6, "tools/list", json!({})).await;
    assert!(tool_names(&tools).contains(&"stalint_watch"));
    assert!(!tool_names(&tools).contains(&"stalint_unwatch"));
    child.kill().await.unwrap();
    child.wait().await.unwrap();
}
