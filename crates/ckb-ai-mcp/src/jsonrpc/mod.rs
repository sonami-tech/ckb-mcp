//! JSON-RPC 2.0 endpoint for plain HTTP requests.
//!
//! This module provides a stateless JSON-RPC interface as an alternative to
//! the SSE-based MCP endpoint, enabling simpler testing and integration.

mod handler;
mod types;

pub use handler::jsonrpc_handler;
