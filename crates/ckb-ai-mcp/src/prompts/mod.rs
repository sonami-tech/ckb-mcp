//! Workflow prompts for guided CKB development tasks.
//!
//! Provides structured prompts that guide AI assistants through
//! common CKB development workflows.

mod definitions;
mod handlers;

pub use definitions::PROMPTS;
pub use handlers::PromptsHandlers;
