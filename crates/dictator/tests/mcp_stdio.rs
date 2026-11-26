//! MCP tool contracts over real stdio, confined to disposable workspaces.

use serde_json::{Value, json};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

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

async fn start_server(workspace: &Path) -> (Child, ChildStdin, BufReader<ChildStdout>) {
    std::fs::write(workspace.join(".dictate.toml"), "[decree.supreme]\n").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_dictator"))
        .arg("mcp")
        .current_dir(workspace)
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
            "clientInfo": {"name": "stdio-regression", "version": "1"}
        }),
    )
    .await;
    input
        .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
        .await
        .unwrap();
    (child, input, output)
}

#[tokio::test]
async fn watch_and_unwatch_swap_the_session_catalog() {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(workspace.path().join("probe.txt"), "clean\n").unwrap();
    let (mut child, mut input, mut output) = start_server(workspace.path()).await;
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

#[tokio::test]
async fn stalint_cursor_reaches_the_remaining_findings() {
    let workspace = tempfile::tempdir().unwrap();
    for file in ["b.txt", "a.txt"] {
        std::fs::write(workspace.path().join(file), "x   \n".repeat(8)).unwrap();
    }
    let (_child, mut input, mut output) = start_server(workspace.path()).await;
    let tools = request(&mut input, &mut output, 2, "tools/list", json!({})).await;
    let stalint = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "stalint")
        .unwrap();
    assert!(stalint["inputSchema"]["properties"].get("cursor").is_some());

    let call = |args: Value| json!({"name": "stalint", "arguments": args});
    let first = request(&mut input, &mut output, 3, "tools/call", call(json!({}))).await;
    let first = &first["structuredContent"]["structuredContent"];
    assert_eq!(first["total"], 16);
    assert_eq!(first["returned"], 10);
    let cursor = first["nextCursor"].as_str().unwrap();
    let last = request(
        &mut input,
        &mut output,
        4,
        "tools/call",
        call(json!({"cursor": cursor})),
    )
    .await;
    let last = &last["structuredContent"]["structuredContent"];
    assert_eq!(last["returned"], 6);
    assert!(last.get("nextCursor").is_none());
    let findings: Vec<_> = [first, last]
        .into_iter()
        .flat_map(|page| page["violations"].as_array().unwrap())
        .map(|v| (v["file"].as_str().unwrap(), v["line"].as_u64().unwrap()))
        .collect();
    let expected: Vec<_> = ["a.txt", "b.txt"]
        .into_iter()
        .flat_map(|file| (1..=8).map(move |line| (file, line)))
        .collect();
    assert_eq!(findings, expected);

    let invalid = request(
        &mut input,
        &mut output,
        5,
        "tools/call",
        call(json!({"cursor": "not-a-cursor"})),
    )
    .await;
    assert_eq!(invalid["isError"], true, "{invalid}");
}
