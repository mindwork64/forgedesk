use serde_json::{json, Value};

use crate::ContextSelection;

/// Read-only MCP STDIO handler. It exposes only the explicitly selected context;
/// there are no tools that can mutate ForgeDesk, GitHub, GitLab or Gitea.
pub fn handle_request(request: &str, context: &ContextSelection) -> Option<String> {
    let request: Value = serde_json::from_str(request).ok()?;
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "forgedesk-context", "version": "0.1.0"}
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": [{
                "name": "forgedesk_context",
                "description": "Return the explicitly selected ForgeDesk issue context.",
                "inputSchema": {"type": "object", "properties": {}}
            }]
        })),
        "tools/call" => {
            let name = request
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if name != "forgedesk_context" {
                Err((-32602, "Unknown tool.".to_string()))
            } else {
                Ok(json!({
                    "content": [{"type": "text", "text": context.prompt()}],
                    "isError": false
                }))
            }
        }
        _ => Err((-32601, "Method not found.".to_string())),
    };
    let payload = match result {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err((code, message)) => json!({
            "jsonrpc": "2.0", "id": id,
            "error": {"code": code, "message": message}
        }),
    };
    serde_json::to_string(&payload).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> ContextSelection {
        ContextSelection {
            issue_key: "FD-1".into(),
            title: "Context".into(),
            description: "Only selected".into(),
            status: "Todo".into(),
            project: None,
            include_description: true,
        }
    }

    #[test]
    fn mcp_exposes_only_read_only_context() {
        let init = handle_request(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
            &context(),
        )
        .expect("init");
        assert!(init.contains("forgedesk-context"));
        let tools = handle_request(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            &context(),
        )
        .expect("tools");
        assert!(tools.contains("forgedesk_context"));
        assert!(!tools.contains("update"));
        let call = handle_request(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"forgedesk_context"}}"#,
            &context(),
        )
        .expect("call");
        assert!(call.contains("FD-1"));
        assert!(call.contains("Only selected"));
    }

    #[test]
    fn mcp_rejects_unknown_tool_and_method() {
        let call = handle_request(
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"write_issue"}}"#,
            &context(),
        )
        .expect("call");
        assert!(call.contains("Unknown tool"));
        let method = handle_request(r#"{"jsonrpc":"2.0","id":5,"method":"delete"}"#, &context())
            .expect("method");
        assert!(method.contains("Method not found"));
    }
}
