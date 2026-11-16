// =============================================================================
// process/mod.rs — Manages the .NET MCP Server child process lifetime
// =============================================================================

use crate::{
    config::Config,
    errors::{McpError, Result},
};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

pub struct McpServerProcess {
    child: Child,
    pub stdin: ChildStdin,
    pub stdout: Lines<BufReader<ChildStdout>>,
    pub stderr: ChildStderr,
}

impl McpServerProcess {
    /// Launch the .NET MCP Server as a child process and return a handle.
    /// Stdout/stdin are piped for MCP protocol messages.
    /// Stderr is piped separately for diagnostic capture.
    pub async fn start(config: &Config) -> Result<Self> {
        tracing::info!(
            "Launching .NET MCP Server from: {}",
            config.dotnet_project_path.display()
        );

        let mut child = Command::new(&config.dotnet_command)
            .arg("run")
            .arg("--no-build")
            .arg("--project")
            .arg(&config.dotnet_project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| McpError::ProcessStart(format!(
                "Failed to start dotnet process: {}. Is .NET 8 installed and was the project already built?", e
            )))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::ProcessStart("Could not capture stdin".to_string()))?;

        let stdout_raw = child
            .stdout
            .take()
            .ok_or_else(|| McpError::ProcessStart("Could not capture stdout".to_string()))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| McpError::ProcessStart("Could not capture stderr".to_string()))?;

        let stdout = BufReader::new(stdout_raw).lines();

        // Brief startup delay so the .NET host can initialise
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        // Check if process already died (config error, missing password, etc.)
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(McpError::ProcessStart(format!(
                    ".NET MCP Server exited immediately with status: {}. \
                     Check appsettings.json for a correct MySQL connection string.",
                    status
                )));
            }
            Ok(None) => {
                tracing::info!(".NET MCP Server is running.");
            }
            Err(e) => {
                return Err(McpError::ProcessStart(format!(
                    "Failed to poll process: {}",
                    e
                )));
            }
        }

        Ok(Self {
            child,
            stdin,
            stdout,
            stderr,
        })
    }

    /// Write a JSON-RPC line to the MCP server's stdin
    pub async fn write_line(&mut self, line: &str) -> Result<()> {
        tracing::debug!("--> SEND: {}", line);
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Read one newline-delimited JSON-RPC response from stdout
    pub async fn read_line(&mut self, timeout_secs: u64) -> Result<String> {
        let deadline = tokio::time::Duration::from_secs(timeout_secs);
        let result = tokio::time::timeout(deadline, self.stdout.next_line()).await;

        match result {
            Ok(Ok(Some(line))) => {
                tracing::debug!("<-- RECV: {}", line);
                Ok(line)
            }
            Ok(Ok(None)) => Err(McpError::ProcessDied),
            Ok(Err(e)) => Err(McpError::Io(e)),
            Err(_) => Err(McpError::Timeout { secs: timeout_secs }),
        }
    }

    /// Check if the child process is still alive
    pub fn is_alive(&mut self) -> bool {
        self.child
            .try_wait()
            .ok()
            .map(|s| s.is_none())
            .unwrap_or(false)
    }

    /// Gracefully terminate the child process
    pub async fn shutdown(&mut self) {
        let _ = self.child.kill().await;
        tracing::info!(".NET MCP Server process terminated.");
    }
}
