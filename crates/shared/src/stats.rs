//! Persistent statistics tracking for MCP servers using redb.
//!
//! This module provides usage statistics that persist across server restarts,
//! helping identify which tools and resources are most frequently used.

use crate::error::{CkbMcpError, Result};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

// Table definitions for redb
const TOOL_CALLS: TableDefinition<&str, u64> = TableDefinition::new("tool_calls");
const TOOL_LAST_CALLED: TableDefinition<&str, u64> = TableDefinition::new("tool_last_called");
const RESOURCE_READS: TableDefinition<&str, u64> = TableDefinition::new("resource_reads");
const RESOURCE_LAST_READ: TableDefinition<&str, u64> = TableDefinition::new("resource_last_read");
const METADATA: TableDefinition<&str, u64> = TableDefinition::new("metadata");

/// Entry for a tool or resource in the stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEntry {
	pub name: String,
	pub count: u64,
	pub last_called: u64,
}

/// Snapshot of all statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSnapshot {
	pub uptime_seconds: u64,
	pub start_time: u64,
	pub total_tool_calls: u64,
	pub total_resource_reads: u64,
	pub total_errors: u64,
	pub tool_calls: Vec<StatsEntry>,
	pub resource_reads: Vec<StatsEntry>,
}

/// Persistent statistics tracker using redb.
pub struct Stats {
	db: Database,
	start_time: u64,
}

impl Stats {
	/// Open or create a stats database at the given path.
	pub fn open(path: impl AsRef<Path>) -> Result<Self> {
		// Create parent directory if it doesn't exist
		if let Some(parent) = path.as_ref().parent() {
			if !parent.as_os_str().is_empty() {
				std::fs::create_dir_all(parent)
					.map_err(|e| CkbMcpError::Internal(format!("Failed to create stats directory: {}", e)))?;
			}
		}

		let db = Database::create(path.as_ref())
			.map_err(|e| CkbMcpError::Internal(format!("Failed to open stats database: {}", e)))?;

		// Initialize tables if they don't exist
		let write_txn = db.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			// Create tables by opening them (redb creates on first access)
			let _ = write_txn.open_table(TOOL_CALLS);
			let _ = write_txn.open_table(TOOL_LAST_CALLED);
			let _ = write_txn.open_table(RESOURCE_READS);
			let _ = write_txn.open_table(RESOURCE_LAST_READ);
			let _ = write_txn.open_table(METADATA);
		}

		write_txn.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit transaction: {}", e)))?;

		let start_time = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		debug!("Stats database opened at {:?}", path.as_ref());

		Ok(Self { db, start_time })
	}

	/// Get current Unix timestamp.
	fn now() -> u64 {
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs()
	}

	/// Record a tool call.
	pub fn record_tool_call(&self, name: &str) {
		if let Err(e) = self.record_tool_call_inner(name) {
			error!("Failed to record tool call: {}", e);
		}
	}

	fn record_tool_call_inner(&self, name: &str) -> Result<()> {
		let write_txn = self.db.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			let mut table = write_txn.open_table(TOOL_CALLS)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let current = table.get(name)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0);

			table.insert(name, current + 1)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			let mut last_table = write_txn.open_table(TOOL_LAST_CALLED)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			last_table.insert(name, Self::now())
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;
		}

		write_txn.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit: {}", e)))?;

		debug!("Recorded tool call: {}", name);
		Ok(())
	}

	/// Record a resource read.
	pub fn record_resource_read(&self, uri: &str) {
		if let Err(e) = self.record_resource_read_inner(uri) {
			error!("Failed to record resource read: {}", e);
		}
	}

	fn record_resource_read_inner(&self, uri: &str) -> Result<()> {
		let write_txn = self.db.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			let mut table = write_txn.open_table(RESOURCE_READS)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let current = table.get(uri)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0);

			table.insert(uri, current + 1)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			let mut last_table = write_txn.open_table(RESOURCE_LAST_READ)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			last_table.insert(uri, Self::now())
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;
		}

		write_txn.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit: {}", e)))?;

		debug!("Recorded resource read: {}", uri);
		Ok(())
	}

	/// Record an error.
	pub fn record_error(&self) {
		if let Err(e) = self.record_error_inner() {
			error!("Failed to record error: {}", e);
		}
	}

	fn record_error_inner(&self) -> Result<()> {
		let write_txn = self.db.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			let mut table = write_txn.open_table(METADATA)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let current = table.get("errors")
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0);

			table.insert("errors", current + 1)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;
		}

		write_txn.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit: {}", e)))?;

		Ok(())
	}

	/// Get a snapshot of all statistics.
	pub fn get_snapshot(&self) -> Result<StatsSnapshot> {
		let read_txn = self.db.begin_read()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		// Read tool calls
		let mut tool_calls = Vec::new();
		let mut total_tool_calls = 0u64;

		if let Ok(table) = read_txn.open_table(TOOL_CALLS) {
			if let Ok(last_table) = read_txn.open_table(TOOL_LAST_CALLED) {
				if let Ok(iter) = table.iter() {
					for entry in iter.flatten() {
						let (name, count) = entry;
						let name = name.value().to_string();
						let count = count.value();
						total_tool_calls += count;

						let last_called = last_table.get(name.as_str())
							.ok()
							.flatten()
							.map(|v| v.value())
							.unwrap_or(0);

						tool_calls.push(StatsEntry { name, count, last_called });
					}
				}
			}
		}

		// Read resource reads
		let mut resource_reads = Vec::new();
		let mut total_resource_reads = 0u64;

		if let Ok(table) = read_txn.open_table(RESOURCE_READS) {
			if let Ok(last_table) = read_txn.open_table(RESOURCE_LAST_READ) {
				if let Ok(iter) = table.iter() {
					for entry in iter.flatten() {
						let (uri, count) = entry;
						let name = uri.value().to_string();
						let count = count.value();
						total_resource_reads += count;

						let last_called = last_table.get(name.as_str())
							.ok()
							.flatten()
							.map(|v| v.value())
							.unwrap_or(0);

						resource_reads.push(StatsEntry { name, count, last_called });
					}
				}
			}
		}

		// Read errors
		let total_errors = if let Ok(table) = read_txn.open_table(METADATA) {
			table.get("errors")
				.ok()
				.flatten()
				.map(|v| v.value())
				.unwrap_or(0)
		} else {
			0
		};

		// Sort by count descending
		tool_calls.sort_by(|a, b| b.count.cmp(&a.count));
		resource_reads.sort_by(|a, b| b.count.cmp(&a.count));

		let now = Self::now();
		let uptime_seconds = now.saturating_sub(self.start_time);

		Ok(StatsSnapshot {
			uptime_seconds,
			start_time: self.start_time,
			total_tool_calls,
			total_resource_reads,
			total_errors,
			tool_calls,
			resource_reads,
		})
	}

	/// Format stats as human-readable text.
	pub fn format_human(&self) -> Result<String> {
		let snapshot = self.get_snapshot()?;
		let mut output = String::new();

		// Header
		output.push_str("CKB MCP Server Stats\n");
		output.push_str("====================\n\n");

		// Uptime
		let days = snapshot.uptime_seconds / 86400;
		let hours = (snapshot.uptime_seconds % 86400) / 3600;
		let minutes = (snapshot.uptime_seconds % 3600) / 60;
		output.push_str(&format!("Uptime: {}d {}h {}m\n", days, hours, minutes));
		output.push_str(&format!("Total Tool Calls: {}\n", snapshot.total_tool_calls));
		output.push_str(&format!("Total Resource Reads: {}\n", snapshot.total_resource_reads));
		output.push_str(&format!("Total Errors: {}\n\n", snapshot.total_errors));

		// Top tools
		if !snapshot.tool_calls.is_empty() {
			output.push_str("Top Tools:\n");
			let now = Self::now();
			for (i, entry) in snapshot.tool_calls.iter().take(10).enumerate() {
				let ago = Self::format_ago(now.saturating_sub(entry.last_called));
				output.push_str(&format!(
					"  {}. {} - {} calls (last: {})\n",
					i + 1, entry.name, entry.count, ago
				));
			}
			output.push('\n');
		}

		// Top resources
		if !snapshot.resource_reads.is_empty() {
			output.push_str("Top Resources:\n");
			let now = Self::now();
			for (i, entry) in snapshot.resource_reads.iter().take(10).enumerate() {
				let ago = Self::format_ago(now.saturating_sub(entry.last_called));
				// Shorten URI for display
				let name = entry.name.replace("ckb-dev-context://", "");
				output.push_str(&format!(
					"  {}. {} - {} reads (last: {})\n",
					i + 1, name, entry.count, ago
				));
			}
		}

		Ok(output)
	}

	/// Format stats as JSON.
	pub fn format_json(&self) -> Result<String> {
		let snapshot = self.get_snapshot()?;
		serde_json::to_string_pretty(&snapshot)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to serialize stats: {}", e)))
	}

	/// Format stats as Prometheus metrics.
	pub fn format_prometheus(&self) -> Result<String> {
		let snapshot = self.get_snapshot()?;
		let mut output = String::new();

		// Tool calls
		output.push_str("# HELP ckb_mcp_tool_calls_total Total tool calls by name\n");
		output.push_str("# TYPE ckb_mcp_tool_calls_total counter\n");
		for entry in &snapshot.tool_calls {
			output.push_str(&format!(
				"ckb_mcp_tool_calls_total{{tool=\"{}\"}} {}\n",
				entry.name, entry.count
			));
		}
		output.push('\n');

		// Resource reads
		output.push_str("# HELP ckb_mcp_resource_reads_total Total resource reads by URI\n");
		output.push_str("# TYPE ckb_mcp_resource_reads_total counter\n");
		for entry in &snapshot.resource_reads {
			// Escape quotes in URI
			let uri = entry.name.replace('"', "\\\"");
			output.push_str(&format!(
				"ckb_mcp_resource_reads_total{{uri=\"{}\"}} {}\n",
				uri, entry.count
			));
		}
		output.push('\n');

		// Errors
		output.push_str("# HELP ckb_mcp_errors_total Total errors\n");
		output.push_str("# TYPE ckb_mcp_errors_total counter\n");
		output.push_str(&format!("ckb_mcp_errors_total {}\n\n", snapshot.total_errors));

		// Uptime
		output.push_str("# HELP ckb_mcp_uptime_seconds Server uptime in seconds\n");
		output.push_str("# TYPE ckb_mcp_uptime_seconds gauge\n");
		output.push_str(&format!("ckb_mcp_uptime_seconds {}\n", snapshot.uptime_seconds));

		Ok(output)
	}

	/// Format a duration as human-readable "X ago" string.
	fn format_ago(seconds: u64) -> String {
		if seconds < 60 {
			format!("{}s ago", seconds)
		} else if seconds < 3600 {
			format!("{}m ago", seconds / 60)
		} else if seconds < 86400 {
			format!("{}h ago", seconds / 3600)
		} else {
			format!("{}d ago", seconds / 86400)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::tempdir;

	#[test]
	fn test_stats_basic() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_stats.redb");

		let stats = Stats::open(&db_path).unwrap();

		// Record some calls
		stats.record_tool_call("get_block");
		stats.record_tool_call("get_block");
		stats.record_tool_call("get_transaction");
		stats.record_resource_read("concepts/cell-model");
		stats.record_error();

		// Get snapshot
		let snapshot = stats.get_snapshot().unwrap();

		assert_eq!(snapshot.total_tool_calls, 3);
		assert_eq!(snapshot.total_resource_reads, 1);
		assert_eq!(snapshot.total_errors, 1);
		assert_eq!(snapshot.tool_calls.len(), 2);
		assert_eq!(snapshot.tool_calls[0].name, "get_block");
		assert_eq!(snapshot.tool_calls[0].count, 2);
	}

	#[test]
	fn test_stats_persistence() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_persist.redb");

		// First session
		{
			let stats = Stats::open(&db_path).unwrap();
			stats.record_tool_call("get_block");
			stats.record_tool_call("get_block");
		}

		// Second session - data should persist
		{
			let stats = Stats::open(&db_path).unwrap();
			stats.record_tool_call("get_block");

			let snapshot = stats.get_snapshot().unwrap();
			assert_eq!(snapshot.total_tool_calls, 3);
		}
	}

	#[test]
	fn test_stats_formats() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_formats.redb");

		let stats = Stats::open(&db_path).unwrap();
		stats.record_tool_call("get_block");

		// Test human format
		let human = stats.format_human().unwrap();
		assert!(human.contains("CKB MCP Server Stats"));
		assert!(human.contains("get_block"));

		// Test JSON format
		let json = stats.format_json().unwrap();
		assert!(json.contains("\"total_tool_calls\""));

		// Test Prometheus format
		let prom = stats.format_prometheus().unwrap();
		assert!(prom.contains("ckb_mcp_tool_calls_total"));
	}
}
