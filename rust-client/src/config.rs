// =============================================================================
// config.rs — Runtime configuration for the Rust MCP Client
// =============================================================================

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Absolute path to the .NET MCP Server project directory
    pub dotnet_project_path: PathBuf,

    /// Binary to use — "dotnet" (run) or a compiled binary path
    pub dotnet_command: String,

    /// Timeout in seconds waiting for MCP responses
    pub mcp_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        // Resolve the dotnet project path relative to our binary
        // so it works from any working directory
        let cargo_manifest = env!("CARGO_MANIFEST_DIR");
        let project_root = PathBuf::from(cargo_manifest)
            .parent()
            .unwrap()
            .to_path_buf();
        let dotnet_path = project_root
            .join("dotnet-mcp-server")
            .join("EmployeeMcpServer");

        Self {
            dotnet_project_path: dotnet_path,
            dotnet_command: "dotnet".to_string(),
            mcp_timeout_secs: 30,
        }
    }
}
