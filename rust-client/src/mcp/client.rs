// =============================================================================
// mcp/client.rs — MCP protocol client: initialize, tools/list, tools/call
// =============================================================================

use crate::{
    config::Config,
    errors::{McpError, Result},
    models::{JsonRpcRequest, JsonRpcResponse, McpTool, ToolExecutionResult},
    process::McpServerProcess,
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

pub struct McpClient {
    process: McpServerProcess,
    timeout_secs: u64,
    pub discovered_tools: Vec<McpTool>,
}

impl McpClient {
    /// Start the .NET process, complete MCP handshake, and discover tools.
    /// This is the single public entry point.
    pub async fn connect(config: &Config) -> Result<Self> {
        let process = McpServerProcess::start(config).await?;
        let mut client = Self {
            process,
            timeout_secs: config.mcp_timeout_secs,
            discovered_tools: Vec::new(),
        };

        client.initialize().await?;
        client.discover_tools().await?;
        Ok(client)
    }

    // -------------------------------------------------------------------------
    // MCP Initialization handshake (Section 6.2 of MCP spec)
    // -------------------------------------------------------------------------
    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Starting MCP initialization handshake...");

        // 1. Send initialize request
        let req = JsonRpcRequest::new(
            next_id(),
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "employee-mcp-client",
                    "version": "1.0.0"
                }
            })),
        );
        self.send_request(&req).await?;

        // 2. Read initialize response
        let resp = self.read_response().await?;
        if let Some(err) = resp.error {
            return Err(McpError::Protocol(format!(
                "MCP initialize failed: {} (code {})",
                err.message, err.code
            )));
        }

        let result = resp
            .result
            .as_ref()
            .ok_or_else(|| McpError::Protocol("No result in initialize response".to_string()))?;

        let protocol_version = result
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        tracing::info!("MCP initialized. Protocol version: {}", protocol_version);

        // 3. Send initialized notification (required by spec)
        let notif = JsonRpcRequest::notification("notifications/initialized", None);
        self.send_request(&notif).await?;

        tracing::info!("MCP handshake complete.");
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Dynamic tool discovery — sends tools/list and stores schemas
    // -------------------------------------------------------------------------
    async fn discover_tools(&mut self) -> Result<()> {
        tracing::info!("Requesting tool catalogue from MCP server...");

        let req = JsonRpcRequest::new(next_id(), "tools/list", Some(json!({})));
        self.send_request(&req).await?;

        let resp = self.read_response().await?;
        if let Some(err) = resp.error {
            return Err(McpError::Protocol(format!(
                "tools/list failed: {}",
                err.message
            )));
        }

        let result = resp
            .result
            .ok_or_else(|| McpError::Protocol("No result in tools/list response".to_string()))?;

        let tools: Vec<McpTool> =
            serde_json::from_value(result.get("tools").cloned().ok_or_else(|| {
                McpError::Protocol("'tools' key missing in tools/list response".to_string())
            })?)
            .map_err(|e| McpError::Protocol(format!("Failed to parse tools list: {}", e)))?;

        tracing::info!("Discovered {} MCP tools:", tools.len());
        for tool in &tools {
            tracing::info!(
                "  - {} : {}",
                tool.name,
                tool.description.as_deref().unwrap_or("(no description)")
            );
        }

        self.discovered_tools = tools;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Execute a discovered MCP tool by name with validated arguments
    // -------------------------------------------------------------------------
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<ToolExecutionResult> {
        // Verify the tool exists in the discovered catalogue
        let tool_exists = self.discovered_tools.iter().any(|t| t.name == tool_name);
        if !tool_exists {
            return Err(McpError::ToolNotFound(format!(
                "'{}' was not found in the discovered tool catalogue. \
                 Available tools: {}",
                tool_name,
                self.discovered_tools
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }

        let req = JsonRpcRequest::new(
            next_id(),
            "tools/call",
            Some(json!({
                "name": tool_name,
                "arguments": arguments
            })),
        );

        self.send_request(&req).await?;
        let resp = self.read_response().await?;

        if let Some(err) = resp.error {
            return Err(McpError::Protocol(format!(
                "Tool '{}' call failed: {}",
                tool_name, err.message
            )));
        }

        let result = resp
            .result
            .ok_or_else(|| McpError::Protocol("No result in tools/call response".to_string()))?;

        // Extract the text content from MCP content array
        let raw_text = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Parse the inner JSON payload returned by our EmployeeService
        let inner: Value = serde_json::from_str(&raw_text).unwrap_or(Value::Null);
        let success = inner
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(!is_error);
        let message = inner
            .get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let data = inner.get("data").cloned();

        Ok(ToolExecutionResult {
            tool_name: tool_name.to_string(),
            success,
            message,
            data,
            raw_response: raw_text,
        })
    }

    // -------------------------------------------------------------------------
    // Low-level helpers
    // -------------------------------------------------------------------------
    async fn send_request(&mut self, req: &JsonRpcRequest) -> Result<()> {
        let line = serde_json::to_string(req)?;
        self.process.write_line(&line).await
    }

    async fn read_response(&mut self) -> Result<JsonRpcResponse> {
        let line = self.process.read_line(self.timeout_secs).await?;
        let resp: JsonRpcResponse = serde_json::from_str(&line).map_err(|e| {
            McpError::Protocol(format!(
                "Failed to parse JSON-RPC response: {}. Raw: {}",
                e, line
            ))
        })?;
        Ok(resp)
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.discovered_tools
            .iter()
            .map(|t| t.name.clone())
            .collect()
    }

    pub async fn shutdown(&mut self) {
        self.process.shutdown().await;
    }
}
