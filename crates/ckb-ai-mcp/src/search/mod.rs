//! Search functionality for tools and resources.
//!
//! Provides tools to search through available MCP tools and documentation
//! resources by keyword matching.

mod handlers;
mod tools;

pub use handlers::SearchHandlers;
pub use tools::SEARCH_TOOLS;
