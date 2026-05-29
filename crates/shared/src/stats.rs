//! Persistent statistics tracking for MCP servers using redb.
//!
//! This module provides usage statistics that persist across server restarts,
//! helping identify which tools and resources are most frequently used.

use crate::error::{CkbMcpError, Result};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

// Table definitions for redb
const TOOL_CALLS: TableDefinition<&str, u64> = TableDefinition::new("tool_calls");
const TOOL_LAST_CALLED: TableDefinition<&str, u64> = TableDefinition::new("tool_last_called");
const RESOURCE_READS: TableDefinition<&str, u64> = TableDefinition::new("resource_reads");
const RESOURCE_LAST_READ: TableDefinition<&str, u64> = TableDefinition::new("resource_last_read");
const RECENT_RESOURCE_READS: TableDefinition<u64, &str> =
	TableDefinition::new("recent_resource_reads");
const METADATA: TableDefinition<&str, u64> = TableDefinition::new("metadata");
const ERROR_COUNTS: TableDefinition<&str, u64> = TableDefinition::new("error_counts");
const ERROR_LAST_SEEN: TableDefinition<&str, u64> = TableDefinition::new("error_last_seen");
const RECENT_RESOURCE_READ_SEQ: &str = "recent_resource_read_seq";
const RECENT_RESOURCE_READ_LIMIT: u64 = 100;

/// Entry for a tool or resource in the stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEntry {
	pub name: String,
	pub count: u64,
	pub last_called: u64,
}

/// Individual recent resource read entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentStatsEntry {
	pub name: String,
	pub timestamp: u64,
	#[serde(skip)]
	sequence: u64,
}

impl RecentStatsEntry {
	fn encode(&self) -> String {
		format!("{}\t{}", self.timestamp, self.name)
	}

	fn decode(sequence: u64, value: &str) -> Option<Self> {
		let (timestamp, name) = value.split_once('\t')?;
		Some(Self {
			name: name.to_string(),
			timestamp: timestamp.parse().ok()?,
			sequence,
		})
	}
}

/// Snapshot of all statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSnapshot {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
	pub uptime_seconds: u64,
	pub start_time: u64,
	pub total_tool_calls: u64,
	pub total_resource_reads: u64,
	pub total_errors: u64,
	pub tool_calls: Vec<StatsEntry>,
	pub resource_reads: Vec<StatsEntry>,
	pub recent_resource_reads: Vec<RecentStatsEntry>,
	pub error_summaries: Vec<StatsEntry>,
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
				std::fs::create_dir_all(parent).map_err(|e| {
					CkbMcpError::Internal(format!("Failed to create stats directory: {}", e))
				})?;
			}
		}

		let db = Database::create(path.as_ref())
			.map_err(|e| CkbMcpError::Internal(format!("Failed to open stats database: {}", e)))?;

		// Initialize tables if they don't exist
		let write_txn = db
			.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			// Create tables by opening them (redb creates on first access)
			let _ = write_txn.open_table(TOOL_CALLS);
			let _ = write_txn.open_table(TOOL_LAST_CALLED);
			let _ = write_txn.open_table(RESOURCE_READS);
			let _ = write_txn.open_table(RESOURCE_LAST_READ);
			let _ = write_txn.open_table(RECENT_RESOURCE_READS);
			let _ = write_txn.open_table(METADATA);
			let _ = write_txn.open_table(ERROR_COUNTS);
			let _ = write_txn.open_table(ERROR_LAST_SEEN);
		}

		write_txn
			.commit()
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
		let write_txn = self
			.db
			.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			let mut table = write_txn
				.open_table(TOOL_CALLS)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let current = table
				.get(name)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0);

			table
				.insert(name, current + 1)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			let mut last_table = write_txn
				.open_table(TOOL_LAST_CALLED)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			last_table
				.insert(name, Self::now())
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;
		}

		write_txn
			.commit()
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
		let now = Self::now();
		let write_txn = self
			.db
			.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			let mut table = write_txn
				.open_table(RESOURCE_READS)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let current = table
				.get(uri)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0);

			table
				.insert(uri, current + 1)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			let mut last_table = write_txn
				.open_table(RESOURCE_LAST_READ)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			last_table
				.insert(uri, now)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			let mut meta_table = write_txn
				.open_table(METADATA)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let seq = meta_table
				.get(RECENT_RESOURCE_READ_SEQ)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0)
				.saturating_add(1);

			meta_table
				.insert(RECENT_RESOURCE_READ_SEQ, seq)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			let recent_entry = RecentStatsEntry {
				name: uri.to_string(),
				timestamp: now,
				sequence: seq,
			}
			.encode();

			let mut recent_table = write_txn
				.open_table(RECENT_RESOURCE_READS)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			recent_table
				.insert(seq, recent_entry.as_str())
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			if seq > RECENT_RESOURCE_READ_LIMIT {
				recent_table
					.remove(seq - RECENT_RESOURCE_READ_LIMIT)
					.map_err(|e| CkbMcpError::Internal(format!("Failed to remove: {}", e)))?;
			}
		}

		write_txn
			.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit: {}", e)))?;

		debug!("Recorded resource read: {}", uri);
		Ok(())
	}

	/// Build a summary key from source name and error message.
	/// Truncates the message to prevent unbounded key growth in the database.
	fn error_summary_key(source: &str, error_msg: &str) -> String {
		let max_msg_len = 120;
		let truncated = if error_msg.len() > max_msg_len {
			let end = error_msg
				.char_indices()
				.take_while(|(i, _)| *i < max_msg_len)
				.last()
				.map(|(i, c)| i + c.len_utf8())
				.unwrap_or(max_msg_len);
			format!("{}...", &error_msg[..end])
		} else {
			error_msg.to_string()
		};
		format!("{}: {}", source, truncated)
	}

	/// Record an error with source and message for per-error tracking.
	pub fn record_error(&self, source: &str, error_msg: &str) {
		if let Err(e) = self.record_error_inner(source, error_msg) {
			error!("Failed to record error: {}", e);
		}
	}

	fn record_error_inner(&self, source: &str, error_msg: &str) -> Result<()> {
		let write_txn = self
			.db
			.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			// Increment aggregate error count.
			let mut meta_table = write_txn
				.open_table(METADATA)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let current = meta_table
				.get("errors")
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0);

			meta_table
				.insert("errors", current + 1)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			// Increment per-error summary count.
			let key = Self::error_summary_key(source, error_msg);

			let mut count_table = write_txn
				.open_table(ERROR_COUNTS)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			let current_count = count_table
				.get(key.as_str())
				.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
				.map(|v| v.value())
				.unwrap_or(0);

			count_table
				.insert(key.as_str(), current_count + 1)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			let mut last_table = write_txn
				.open_table(ERROR_LAST_SEEN)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

			last_table
				.insert(key.as_str(), Self::now())
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;
		}

		write_txn
			.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit: {}", e)))?;

		debug!("Recorded error: {} - {}", source, error_msg);
		Ok(())
	}

	/// Get a snapshot of all statistics.
	pub fn get_snapshot(&self) -> Result<StatsSnapshot> {
		let read_txn = self
			.db
			.begin_read()
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

						let last_called = last_table
							.get(name.as_str())
							.ok()
							.flatten()
							.map(|v| v.value())
							.unwrap_or(0);

						tool_calls.push(StatsEntry {
							name,
							count,
							last_called,
						});
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

						let last_called = last_table
							.get(name.as_str())
							.ok()
							.flatten()
							.map(|v| v.value())
							.unwrap_or(0);

						resource_reads.push(StatsEntry {
							name,
							count,
							last_called,
						});
					}
				}
			}
		}

		// Read errors
		let total_errors = if let Ok(table) = read_txn.open_table(METADATA) {
			table
				.get("errors")
				.ok()
				.flatten()
				.map(|v| v.value())
				.unwrap_or(0)
		} else {
			0
		};

		// Read recent resource reads
		let mut recent_resource_reads = Vec::new();

		if let Ok(table) = read_txn.open_table(RECENT_RESOURCE_READS) {
			if let Ok(iter) = table.iter() {
				for entry in iter.flatten() {
					let (seq, value) = entry;
					if let Some(entry) = RecentStatsEntry::decode(seq.value(), value.value()) {
						recent_resource_reads.push(entry);
					}
				}
			}
		}

		// Read error summaries
		let mut error_summaries = Vec::new();

		if let Ok(table) = read_txn.open_table(ERROR_COUNTS) {
			if let Ok(last_table) = read_txn.open_table(ERROR_LAST_SEEN) {
				if let Ok(iter) = table.iter() {
					for entry in iter.flatten() {
						let (key, count) = entry;
						let name = key.value().to_string();
						let count = count.value();

						let last_called = last_table
							.get(name.as_str())
							.ok()
							.flatten()
							.map(|v| v.value())
							.unwrap_or(0);

						error_summaries.push(StatsEntry {
							name,
							count,
							last_called,
						});
					}
				}
			}
		}

		// Sort by count descending
		tool_calls.sort_by_key(|b| std::cmp::Reverse(b.count));
		resource_reads.sort_by(|a, b| {
			b.count
				.cmp(&a.count)
				.then_with(|| b.last_called.cmp(&a.last_called))
				.then_with(|| a.name.cmp(&b.name))
		});
		recent_resource_reads.sort_by_key(|b| std::cmp::Reverse(b.sequence));
		error_summaries.sort_by_key(|b| std::cmp::Reverse(b.count));

		let now = Self::now();
		let uptime_seconds = now.saturating_sub(self.start_time);

		Ok(StatsSnapshot {
			version: None,
			uptime_seconds,
			start_time: self.start_time,
			total_tool_calls,
			total_resource_reads,
			total_errors,
			tool_calls,
			resource_reads,
			recent_resource_reads,
			error_summaries,
		})
	}

	/// Format stats as human-readable text.
	pub fn format_human(&self, version: Option<&str>) -> Result<String> {
		let snapshot = self.get_snapshot()?;
		let mut output = String::new();

		// Header
		match version {
			Some(v) => {
				let header = format!("CKB MCP Server Stats (v{})", v);
				output.push_str(&header);
				output.push('\n');
				output.push_str(&"=".repeat(header.len()));
				output.push_str("\n\n");
			}
			None => {
				output.push_str("CKB MCP Server Stats\n");
				output.push_str("====================\n\n");
			}
		}

		// Uptime
		let days = snapshot.uptime_seconds / 86400;
		let hours = (snapshot.uptime_seconds % 86400) / 3600;
		let minutes = (snapshot.uptime_seconds % 3600) / 60;
		output.push_str(&format!("Uptime: {}d {}h {}m\n", days, hours, minutes));
		output.push_str(&format!(
			"Total Tool Calls: {}\n",
			snapshot.total_tool_calls
		));
		output.push_str(&format!(
			"Total Resource Reads: {}\n",
			snapshot.total_resource_reads
		));
		output.push_str(&format!("Total Errors: {}\n\n", snapshot.total_errors));

		// Top tools
		if !snapshot.tool_calls.is_empty() {
			output.push_str("Top Tools:\n");
			let now = Self::now();
			for (i, entry) in snapshot.tool_calls.iter().take(10).enumerate() {
				let ago = Self::format_ago(now.saturating_sub(entry.last_called));
				output.push_str(&format!(
					"  {}. {} - {} calls (last: {})\n",
					i + 1,
					entry.name,
					entry.count,
					ago
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
					i + 1,
					name,
					entry.count,
					ago
				));
			}
			output.push('\n');
		}

		// Recent resource read requests
		if !snapshot.recent_resource_reads.is_empty() {
			output.push_str("Recent Requests:\n");
			let now = Self::now();
			for (i, entry) in snapshot.recent_resource_reads.iter().take(10).enumerate() {
				let ago = Self::format_ago(now.saturating_sub(entry.timestamp));
				let name = entry.name.replace("ckb-dev-context://", "");
				output.push_str(&format!("  {}. {} - {}\n", i + 1, name, ago));
			}
			output.push('\n');
		}

		// Top errors
		if !snapshot.error_summaries.is_empty() {
			output.push_str("Top Errors:\n");
			let now = Self::now();
			for (i, entry) in snapshot.error_summaries.iter().take(10).enumerate() {
				let ago = Self::format_ago(now.saturating_sub(entry.last_called));
				output.push_str(&format!(
					"  {}. {} - {} errors (last: {})\n",
					i + 1,
					entry.name,
					entry.count,
					ago
				));
			}
		}

		Ok(output)
	}

	/// Format stats as JSON.
	pub fn format_json(&self, version: Option<&str>) -> Result<String> {
		let mut snapshot = self.get_snapshot()?;
		snapshot.version = version.map(|v| v.to_string());
		serde_json::to_string_pretty(&snapshot)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to serialize stats: {}", e)))
	}

	/// Format stats as Prometheus metrics.
	pub fn format_prometheus(&self, version: Option<&str>) -> Result<String> {
		let snapshot = self.get_snapshot()?;
		let mut output = String::new();

		// Server info (with version)
		if let Some(v) = version {
			output.push_str("# HELP ckb_mcp_info Server information\n");
			output.push_str("# TYPE ckb_mcp_info gauge\n");
			output.push_str(&format!("ckb_mcp_info{{version=\"{}\"}} 1\n\n", v));
		}

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
		output.push_str(&format!(
			"ckb_mcp_errors_total {}\n\n",
			snapshot.total_errors
		));

		// Error summaries
		if !snapshot.error_summaries.is_empty() {
			output
				.push_str("# HELP ckb_mcp_error_summary_total Error count by source and message\n");
			output.push_str("# TYPE ckb_mcp_error_summary_total counter\n");
			for entry in snapshot.error_summaries.iter().take(10) {
				let label = entry
					.name
					.replace('\\', "\\\\")
					.replace('"', "\\\"")
					.replace('\n', "\\n");
				output.push_str(&format!(
					"ckb_mcp_error_summary_total{{error=\"{}\"}} {}\n",
					label, entry.count
				));
			}
			output.push('\n');
		}

		// Uptime
		output.push_str("# HELP ckb_mcp_uptime_seconds Server uptime in seconds\n");
		output.push_str("# TYPE ckb_mcp_uptime_seconds gauge\n");
		output.push_str(&format!(
			"ckb_mcp_uptime_seconds {}\n",
			snapshot.uptime_seconds
		));

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
		stats.record_error("get_block", "Connection refused");

		// Get snapshot
		let snapshot = stats.get_snapshot().unwrap();

		assert_eq!(snapshot.total_tool_calls, 3);
		assert_eq!(snapshot.total_resource_reads, 1);
		assert_eq!(snapshot.total_errors, 1);
		assert_eq!(snapshot.tool_calls.len(), 2);
		assert_eq!(snapshot.tool_calls[0].name, "get_block");
		assert_eq!(snapshot.tool_calls[0].count, 2);
		assert_eq!(snapshot.error_summaries.len(), 1);
		assert_eq!(
			snapshot.error_summaries[0].name,
			"get_block: Connection refused"
		);
		assert_eq!(snapshot.error_summaries[0].count, 1);
	}

	#[test]
	fn test_resource_reads_sort_by_count_then_recency() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_resource_sorting.redb");

		let stats = Stats::open(&db_path).unwrap();

		stats.record_resource_read("ckb://docs/older");
		std::thread::sleep(std::time::Duration::from_secs(1));
		stats.record_resource_read("ckb://docs/newer");
		stats.record_resource_read("ckb://docs/top");
		stats.record_resource_read("ckb://docs/top");

		let snapshot = stats.get_snapshot().unwrap();
		assert_eq!(snapshot.resource_reads[0].name, "ckb://docs/top");
		assert_eq!(snapshot.resource_reads[0].count, 2);
		assert_eq!(snapshot.resource_reads[1].name, "ckb://docs/newer");
		assert_eq!(snapshot.resource_reads[2].name, "ckb://docs/older");
	}

	#[test]
	fn test_recent_resource_reads_keep_request_order_and_duplicates() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_recent_resource_reads.redb");

		let stats = Stats::open(&db_path).unwrap();

		stats.record_resource_read("ckb://docs/first");
		stats.record_resource_read("ckb://docs/second");
		stats.record_resource_read("ckb://docs/second");

		let snapshot = stats.get_snapshot().unwrap();
		let names: Vec<&str> = snapshot
			.recent_resource_reads
			.iter()
			.map(|entry| entry.name.as_str())
			.collect();

		assert_eq!(
			names,
			vec!["ckb://docs/second", "ckb://docs/second", "ckb://docs/first"]
		);
	}

	#[test]
	fn test_human_stats_show_recent_requests_after_top_resources() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_recent_requests_human.redb");

		let stats = Stats::open(&db_path).unwrap();

		for i in 0..12 {
			stats.record_resource_read(&format!("ckb://docs/request-{}", i));
		}

		let human = stats.format_human(None).unwrap();
		let top_index = human
			.find("Top Resources:")
			.expect("human stats should include top resources");
		let recent_index = human
			.find("Recent Requests:")
			.expect("human stats should include recent requests");

		assert!(
			recent_index > top_index,
			"Recent Requests should render below Top Resources"
		);
		assert!(human.contains("1. ckb://docs/request-11 - "));
		assert!(human.contains("10. ckb://docs/request-2 - "));
		assert!(!human.contains("11. ckb://docs/request-1 - "));
		assert!(human.contains("ago"));
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
		stats.record_error("get_block", "CKB RPC error: timeout");

		// Test human format without version
		let human = stats.format_human(None).unwrap();
		assert!(human.contains("CKB MCP Server Stats"));
		assert!(human.contains("get_block"));
		assert!(human.contains("Top Errors:"));
		assert!(human.contains("get_block: CKB RPC error: timeout"));

		// Test human format with version
		let human_v = stats.format_human(Some("1.5.0")).unwrap();
		assert!(human_v.contains("CKB MCP Server Stats (v1.5.0)"));

		// Test JSON format without version
		let json = stats.format_json(None).unwrap();
		assert!(json.contains("\"total_tool_calls\""));
		assert!(json.contains("\"error_summaries\""));
		assert!(!json.contains("\"version\""));

		// Test JSON format with version
		let json_v = stats.format_json(Some("1.5.0")).unwrap();
		assert!(json_v.contains("\"version\": \"1.5.0\""));

		// Test Prometheus format without version
		let prom = stats.format_prometheus(None).unwrap();
		assert!(prom.contains("ckb_mcp_tool_calls_total"));
		assert!(prom.contains("ckb_mcp_error_summary_total"));
		assert!(!prom.contains("ckb_mcp_info"));

		// Test Prometheus format with version
		let prom_v = stats.format_prometheus(Some("1.5.0")).unwrap();
		assert!(prom_v.contains("ckb_mcp_info{version=\"1.5.0\"} 1"));
	}

	#[test]
	fn test_stats_error_summaries() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_error_summaries.redb");

		let stats = Stats::open(&db_path).unwrap();

		// Record multiple errors with same source+message
		stats.record_error("rpc_get_block", "Connection refused");
		stats.record_error("rpc_get_block", "Connection refused");
		stats.record_error("rpc_get_block", "Connection refused");

		// Record different error for same source
		stats.record_error("rpc_get_block", "Timeout");

		// Record error for different source
		stats.record_error("dev_deploy", "Insufficient balance");

		let snapshot = stats.get_snapshot().unwrap();

		assert_eq!(snapshot.total_errors, 5);
		assert_eq!(snapshot.error_summaries.len(), 3);

		// Sorted by count descending
		assert_eq!(snapshot.error_summaries[0].count, 3);
		assert!(snapshot.error_summaries[0]
			.name
			.contains("Connection refused"));
		assert_eq!(snapshot.error_summaries[1].count, 1);
		assert_eq!(snapshot.error_summaries[2].count, 1);
	}

	#[test]
	fn test_stats_error_truncation() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_error_truncation.redb");

		let stats = Stats::open(&db_path).unwrap();

		let long_msg = "x".repeat(300);
		stats.record_error("some_tool", &long_msg);

		let snapshot = stats.get_snapshot().unwrap();
		assert_eq!(snapshot.error_summaries.len(), 1);
		assert!(snapshot.error_summaries[0].name.len() < 150);
		assert!(snapshot.error_summaries[0].name.ends_with("..."));
	}

	#[test]
	fn test_stats_error_persistence() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_error_persist.redb");

		{
			let stats = Stats::open(&db_path).unwrap();
			stats.record_error("tool_a", "Some error");
			stats.record_error("tool_a", "Some error");
		}

		{
			let stats = Stats::open(&db_path).unwrap();
			stats.record_error("tool_a", "Some error");

			let snapshot = stats.get_snapshot().unwrap();
			assert_eq!(snapshot.total_errors, 3);
			assert_eq!(snapshot.error_summaries.len(), 1);
			assert_eq!(snapshot.error_summaries[0].count, 3);
		}
	}
}
