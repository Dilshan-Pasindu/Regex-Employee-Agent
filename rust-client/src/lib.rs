// =============================================================================
// lib.rs — Employee Management MCP Client Library
// Exposes modules for CLI, tests, and future HTTP services
// =============================================================================

#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_unwrap_or_default)]

pub mod config;
pub mod confirmation;
pub mod context;
pub mod errors;
pub mod intent;
pub mod mcp;
pub mod models;
pub mod planner;
pub mod process;
pub mod response;
pub mod server;
pub mod suggestions;
