//! CKB composite tools module for high-level blockchain operations.
//!
//! This module provides 5 composite tools that combine multiple RPC calls:
//! - ckb_query_address: Get complete address state
//! - ckb_query_chain_status: Get chain health snapshot
//! - ckb_query_transaction: Get transaction with resolved inputs
//! - ckb_validate_transaction: Pre-submission validation
//! - ckb_query_script_cells: Find cells by script

mod handlers;
mod tools;

pub use handlers::CkbHandlers;
pub use tools::CKB_TOOLS;
