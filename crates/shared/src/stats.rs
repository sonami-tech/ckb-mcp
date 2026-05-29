//! Persistent statistics tracking for MCP servers using redb.
//!
//! This module provides usage statistics that persist across server restarts,
//! helping identify which tools and resources are most frequently used.

use crate::error::{CkbMcpError, Result};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, warn};

// Table definitions for redb
const TOOL_CALLS: TableDefinition<&str, u64> = TableDefinition::new("tool_calls");
const TOOL_LAST_CALLED: TableDefinition<&str, u64> = TableDefinition::new("tool_last_called");
const RESOURCE_READS: TableDefinition<&str, u64> = TableDefinition::new("resource_reads");
const RESOURCE_LAST_READ: TableDefinition<&str, u64> = TableDefinition::new("resource_last_read");
// Unified time-ordered feed of every intentional request (tool calls, resource
// reads, prompt gets, and list/discovery calls). Distinct table name from the
// former resource-only `recent_resource_reads` so the format change starts the
// feed fresh rather than decoding incompatible old rows.
const RECENT_REQUESTS: TableDefinition<u64, &str> = TableDefinition::new("recent_requests");
const METADATA: TableDefinition<&str, u64> = TableDefinition::new("metadata");
const ERROR_COUNTS: TableDefinition<&str, u64> = TableDefinition::new("error_counts");
const ERROR_LAST_SEEN: TableDefinition<&str, u64> = TableDefinition::new("error_last_seen");
const RECENT_REQUEST_SEQ: &str = "recent_request_seq";
const RECENT_REQUEST_LIMIT: u64 = 100;
// Time-ordered feed of recent failed requests, kept independent from the
// success feed so an error burst cannot evict recent successful requests
// (and vice-versa). Each entry carries the error message in its detail field.
const RECENT_ERRORS: TableDefinition<u64, &str> = TableDefinition::new("recent_errors");
const RECENT_ERROR_SEQ: &str = "recent_error_seq";
const RECENT_ERROR_LIMIT: u64 = 100;

/// What to do when the on-disk stats database is incompatible or corrupt.
///
/// The stats database holds only usage telemetry, never source-of-truth data,
/// so the default is to discard and recreate it rather than crash-loop on an
/// upgrade that changed the format. Operators who would rather fail loudly and
/// inspect the file can choose [`OnIncompatible::Fail`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnIncompatible {
	/// Delete the incompatible/corrupt file and start fresh (default).
	#[default]
	Reset,
	/// Leave the file untouched and return an error.
	Fail,
}

/// Entry for a tool or resource in the stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEntry {
	pub name: String,
	pub count: u64,
	pub last_called: u64,
}

/// Individual entry in a recent-events feed (a request or a failed request).
///
/// `kind` tags the event type for display, e.g. `tool`, `resource`, `prompt`,
/// or `list` in the request feed; `error` entries also carry a `detail`
/// message (empty for successful requests).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentStatsEntry {
	pub kind: String,
	pub name: String,
	pub timestamp: u64,
	#[serde(skip_serializing_if = "String::is_empty", default)]
	pub detail: String,
	#[serde(skip)]
	sequence: u64,
}

impl RecentStatsEntry {
	/// Encode as tab-separated `timestamp\tkind\tname\tdetail`. The detail field
	/// is last so it can contain spaces; tabs are stripped from it to keep the
	/// delimiter unambiguous.
	fn encode(&self) -> String {
		format!(
			"{}\t{}\t{}\t{}",
			self.timestamp,
			self.kind,
			self.name,
			self.detail.replace('\t', " ")
		)
	}

	fn decode(sequence: u64, value: &str) -> Option<Self> {
		// timestamp \t kind \t name \t detail(optional, may be empty)
		let mut parts = value.splitn(4, '\t');
		let timestamp = parts.next()?.parse().ok()?;
		let kind = parts.next()?.to_string();
		let name = parts.next()?.to_string();
		let detail = parts.next().unwrap_or("").to_string();
		Some(Self {
			kind,
			name,
			timestamp,
			detail,
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
	pub recent_requests: Vec<RecentStatsEntry>,
	pub recent_errors: Vec<RecentStatsEntry>,
	pub error_summaries: Vec<StatsEntry>,
}

/// Persistent statistics tracker using redb.
pub struct Stats {
	db: Database,
	start_time: u64,
}

impl Stats {
	/// Open or create a stats database at the given path, using the default
	/// incompatibility policy ([`OnIncompatible::Reset`]).
	pub fn open(path: impl AsRef<Path>) -> Result<Self> {
		Self::open_with_policy(path, OnIncompatible::default())
	}

	/// Open or create a stats database, choosing what to do if the existing
	/// file is incompatible or corrupt.
	///
	/// "Incompatible or corrupt" is detected in two stages:
	/// 1. The file fails to open at all — redb validates its format version and
	///    structural integrity (checksums) on open, so a foreign/newer format
	///    or a truncated/garbled file surfaces here.
	/// 2. The file opens, but a known table's stored key/value types do not
	///    match this build's schema — redb is type-checked on `open_table`, so
	///    a stale layout from an older build surfaces as a type mismatch.
	///
	/// On detection: with [`OnIncompatible::Reset`] the file is deleted and a
	/// fresh database created; with [`OnIncompatible::Fail`] the file is left
	/// untouched and an error is returned. Transient errors (e.g. the parent
	/// directory being unwritable) are never treated as incompatibility and
	/// never trigger deletion.
	pub fn open_with_policy(path: impl AsRef<Path>, on_incompatible: OnIncompatible) -> Result<Self> {
		let path = path.as_ref();

		// Create parent directory if it doesn't exist
		if let Some(parent) = path.parent() {
			if !parent.as_os_str().is_empty() {
				std::fs::create_dir_all(parent).map_err(|e| {
					CkbMcpError::Internal(format!("Failed to create stats directory: {}", e))
				})?;
			}
		}

		let db = match Self::try_open_and_probe(path) {
			Ok(db) => db,
			Err(reason) => match on_incompatible {
				OnIncompatible::Reset => {
					warn!(
						path = %path.display(),
						reason = %reason,
						"Stats database is incompatible or corrupt; deleting it and starting fresh \
						 (telemetry only, no source-of-truth data is lost). Use \
						 --no-reset-stats-on-incompatible to keep the file and fail instead."
					);
					// Only remove an existing regular file; never follow into a
					// directory or act on a path that isn't there.
					if path.is_file() {
						std::fs::remove_file(path).map_err(|e| {
							CkbMcpError::Internal(format!(
								"Failed to remove incompatible stats database {}: {}",
								path.display(),
								e
							))
						})?;
					}
					Database::create(path).map_err(|e| {
						CkbMcpError::Internal(format!("Failed to recreate stats database: {}", e))
					})?
				}
				OnIncompatible::Fail => {
					return Err(CkbMcpError::Internal(format!(
						"Stats database at {} is incompatible or corrupt ({}). Refusing to delete it \
						 because --no-reset-stats-on-incompatible is set. Inspect or remove the file \
						 manually, or drop the flag to reset it automatically.",
						path.display(),
						reason
					)));
				}
			},
		};

		// Ensure all tables exist (redb creates them on first access).
		let write_txn = db
			.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		{
			let _ = write_txn.open_table(TOOL_CALLS);
			let _ = write_txn.open_table(TOOL_LAST_CALLED);
			let _ = write_txn.open_table(RESOURCE_READS);
			let _ = write_txn.open_table(RESOURCE_LAST_READ);
			let _ = write_txn.open_table(RECENT_REQUESTS);
			let _ = write_txn.open_table(RECENT_ERRORS);
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

		debug!("Stats database opened at {:?}", path);

		Ok(Self { db, start_time })
	}

	/// Attempt to open the database and verify its schema is compatible.
	///
	/// Returns `Ok(db)` if the file opens and every known table either is absent
	/// (a fresh or partially-initialised DB) or carries the expected types.
	/// Returns `Err(reason)` describing the incompatibility/corruption if the
	/// file cannot be opened or a table's types do not match this schema. The
	/// `reason` string is for logging only; the caller decides what to do.
	fn try_open_and_probe(path: &Path) -> std::result::Result<Database, String> {
		// Stage 1: open. redb validates format version + integrity here.
		let db = Database::create(path).map_err(|e| format!("cannot open database: {}", e))?;

		// Stage 2: type-check the schema by opening each table inside a read
		// transaction. A read transaction does NOT create missing tables, so an
		// absent table (TableDoesNotExist) is fine — it just means this is a new
		// or not-yet-initialised DB. A type mismatch, however, means the on-disk
		// layout is from an incompatible build.
		let read_txn = db
			.begin_read()
			.map_err(|e| format!("cannot begin read transaction: {}", e))?;

		Self::probe_table(&read_txn, TOOL_CALLS, "tool_calls")?;
		Self::probe_table(&read_txn, TOOL_LAST_CALLED, "tool_last_called")?;
		Self::probe_table(&read_txn, RESOURCE_READS, "resource_reads")?;
		Self::probe_table(&read_txn, RESOURCE_LAST_READ, "resource_last_read")?;
		Self::probe_recent_table(&read_txn, RECENT_REQUESTS, "recent_requests")?;
		Self::probe_recent_table(&read_txn, RECENT_ERRORS, "recent_errors")?;
		Self::probe_table(&read_txn, METADATA, "metadata")?;
		Self::probe_table(&read_txn, ERROR_COUNTS, "error_counts")?;
		Self::probe_table(&read_txn, ERROR_LAST_SEEN, "error_last_seen")?;

		drop(read_txn);
		Ok(db)
	}

	/// Read-probe one `<&str, u64>` table: absent is OK, wrong type is fatal.
	fn probe_table(
		read_txn: &redb::ReadTransaction,
		table_def: TableDefinition<&'static str, u64>,
		name: &str,
	) -> std::result::Result<(), String> {
		match read_txn.open_table(table_def) {
			Ok(_) => Ok(()),
			Err(redb::TableError::TableDoesNotExist(_)) => Ok(()),
			Err(e) => Err(format!("table `{}` is incompatible: {}", name, e)),
		}
	}

	/// Read-probe one `<u64, &str>` recent-feed table: absent OK, wrong type fatal.
	fn probe_recent_table(
		read_txn: &redb::ReadTransaction,
		table_def: TableDefinition<u64, &'static str>,
		name: &str,
	) -> std::result::Result<(), String> {
		match read_txn.open_table(table_def) {
			Ok(_) => Ok(()),
			Err(redb::TableError::TableDoesNotExist(_)) => Ok(()),
			Err(e) => Err(format!("table `{}` is incompatible: {}", name, e)),
		}
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
		let now = Self::now();
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
				.insert(name, now)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

			// A prompt get arrives here as `prompt:NAME`; tag it `prompt` in the
			// feed while leaving the counter key unchanged. Everything else is a
			// genuine tool call.
			let (kind, feed_name) = match name.strip_prefix("prompt:") {
				Some(rest) => ("prompt", rest),
				None => ("tool", name),
			};
			Self::append_recent(
				&write_txn,
				RECENT_REQUESTS,
				RECENT_REQUEST_SEQ,
				RECENT_REQUEST_LIMIT,
				kind,
				feed_name,
				"",
				now,
			)?;
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

			Self::append_recent(
				&write_txn,
				RECENT_REQUESTS,
				RECENT_REQUEST_SEQ,
				RECENT_REQUEST_LIMIT,
				"resource",
				uri,
				"",
				now,
			)?;
		}

		write_txn
			.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit: {}", e)))?;

		debug!("Recorded resource read: {}", uri);
		Ok(())
	}

	/// Append one entry to a recent-events ring buffer within an open write
	/// transaction. Bumps the buffer's sequence counter (stored in METADATA),
	/// inserts the encoded entry keyed by sequence, and evicts the oldest entry
	/// once the buffer exceeds `limit`. Shared by the request feed and the
	/// failed-request feed so both behave identically.
	#[allow(clippy::too_many_arguments)]
	fn append_recent(
		write_txn: &redb::WriteTransaction,
		table_def: TableDefinition<u64, &'static str>,
		seq_key: &str,
		limit: u64,
		kind: &str,
		name: &str,
		detail: &str,
		now: u64,
	) -> Result<()> {
		let mut meta_table = write_txn
			.open_table(METADATA)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

		let seq = meta_table
			.get(seq_key)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to read: {}", e)))?
			.map(|v| v.value())
			.unwrap_or(0)
			.saturating_add(1);

		meta_table
			.insert(seq_key, seq)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

		let encoded = RecentStatsEntry {
			kind: kind.to_string(),
			name: name.to_string(),
			timestamp: now,
			detail: detail.to_string(),
			sequence: seq,
		}
		.encode();

		let mut recent_table = write_txn
			.open_table(table_def)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to open table: {}", e)))?;

		recent_table
			.insert(seq, encoded.as_str())
			.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;

		if seq > limit {
			recent_table
				.remove(seq - limit)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to remove: {}", e)))?;
		}

		Ok(())
	}

	/// Record an intentional request that has no per-item counter of its own
	/// (the list/discovery calls: `tools/list`, `resources/list`,
	/// `prompts/list`). These appear only in the recent-requests feed, never in
	/// Top Tools or Top Resources.
	pub fn record_request(&self, kind: &str, name: &str) {
		if let Err(e) = self.record_request_inner(kind, name) {
			error!("Failed to record request: {}", e);
		}
	}

	fn record_request_inner(&self, kind: &str, name: &str) -> Result<()> {
		let now = Self::now();
		let write_txn = self
			.db
			.begin_write()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to begin transaction: {}", e)))?;

		Self::append_recent(
			&write_txn,
			RECENT_REQUESTS,
			RECENT_REQUEST_SEQ,
			RECENT_REQUEST_LIMIT,
			kind,
			name,
			"",
			now,
		)?;

		write_txn
			.commit()
			.map_err(|e| CkbMcpError::Internal(format!("Failed to commit: {}", e)))?;

		debug!("Recorded request: {} {}", kind, name);
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

	/// Record a failed request. `kind` tags the request type (tool, resource,
	/// prompt, list) for the recent-failures feed; `source` is the target
	/// (tool name / URI / prompt name) and `error_msg` the failure message.
	/// Feeds both the aggregate Top Errors summary and the recent-failures feed.
	pub fn record_error(&self, kind: &str, source: &str, error_msg: &str) {
		if let Err(e) = self.record_error_inner(kind, source, error_msg) {
			error!("Failed to record error: {}", e);
		}
	}

	fn record_error_inner(&self, kind: &str, source: &str, error_msg: &str) -> Result<()> {
		let now = Self::now();
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
				.insert(key.as_str(), now)
				.map_err(|e| CkbMcpError::Internal(format!("Failed to insert: {}", e)))?;
		}

		// Append to the recent-failures feed AFTER the block above closes, so the
		// METADATA table guard is dropped first: redb forbids opening the same
		// table twice within one transaction, and append_recent reopens METADATA
		// for its sequence counter.
		Self::append_recent(
			&write_txn,
			RECENT_ERRORS,
			RECENT_ERROR_SEQ,
			RECENT_ERROR_LIMIT,
			kind,
			source,
			error_msg,
			now,
		)?;

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

		// Read both recent-events feeds (successes and failures).
		let mut recent_requests = Self::read_recent(&read_txn, RECENT_REQUESTS);
		let mut recent_errors = Self::read_recent(&read_txn, RECENT_ERRORS);

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
		recent_requests.sort_by_key(|b| std::cmp::Reverse(b.sequence));
		recent_errors.sort_by_key(|b| std::cmp::Reverse(b.sequence));
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
			recent_requests,
			recent_errors,
			error_summaries,
		})
	}

	/// Read and decode all entries from a recent-events ring-buffer table.
	/// Returns them unsorted; callers sort by sequence for newest-first order.
	fn read_recent(
		read_txn: &redb::ReadTransaction,
		table_def: TableDefinition<u64, &'static str>,
	) -> Vec<RecentStatsEntry> {
		let mut entries = Vec::new();
		if let Ok(table) = read_txn.open_table(table_def) {
			if let Ok(iter) = table.iter() {
				for entry in iter.flatten() {
					let (seq, value) = entry;
					if let Some(decoded) = RecentStatsEntry::decode(seq.value(), value.value()) {
						entries.push(decoded);
					}
				}
			}
		}
		entries
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

		// Recent requests (every intentional hit: tools, resources, prompts, lists)
		if !snapshot.recent_requests.is_empty() {
			output.push_str("Recent Requests:\n");
			let now = Self::now();
			for (i, entry) in snapshot.recent_requests.iter().take(10).enumerate() {
				let ago = Self::format_ago(now.saturating_sub(entry.timestamp));
				let name = entry.name.replace("ckb-dev-context://", "");
				output.push_str(&format!("  {}. [{}] {} - {}\n", i + 1, entry.kind, name, ago));
			}
			output.push('\n');
		}

		// Recent failed requests (time-ordered failures; complements Top Errors)
		if !snapshot.recent_errors.is_empty() {
			output.push_str("Recent Failed Requests:\n");
			let now = Self::now();
			for (i, entry) in snapshot.recent_errors.iter().take(10).enumerate() {
				let ago = Self::format_ago(now.saturating_sub(entry.timestamp));
				let name = entry.name.replace("ckb-dev-context://", "");
				if entry.detail.is_empty() {
					output.push_str(&format!("  {}. [{}] {} - {}\n", i + 1, entry.kind, name, ago));
				} else {
					output.push_str(&format!(
						"  {}. [{}] {} - {} ({})\n",
						i + 1,
						entry.kind,
						name,
						entry.detail,
						ago
					));
				}
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
		stats.record_error("tool", "get_block", "Connection refused");

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
	fn test_recent_requests_keep_request_order_and_duplicates() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_recent_requests.redb");

		let stats = Stats::open(&db_path).unwrap();

		stats.record_resource_read("ckb://docs/first");
		stats.record_resource_read("ckb://docs/second");
		stats.record_resource_read("ckb://docs/second");

		let snapshot = stats.get_snapshot().unwrap();
		let names: Vec<&str> = snapshot
			.recent_requests
			.iter()
			.map(|entry| entry.name.as_str())
			.collect();

		// Newest-first, duplicates preserved (it is a feed, not a set).
		assert_eq!(
			names,
			vec!["ckb://docs/second", "ckb://docs/second", "ckb://docs/first"]
		);
		// Resource reads are tagged `resource` in the unified feed.
		assert!(
			snapshot.recent_requests.iter().all(|e| e.kind == "resource"),
			"resource reads must carry kind=resource in the feed"
		);
	}

	#[test]
	fn test_recent_requests_unifies_all_request_kinds_in_order() {
		// The "Recent Requests" feed must reflect EVERY intentional hit, not just
		// resource reads — that is the whole point of the unified feed. A tool
		// call, a resource read, a prompt get, and a list call must all appear,
		// interleaved newest-first and tagged by kind.
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_unified_feed.redb");
		let stats = Stats::open(&db_path).unwrap();

		stats.record_tool_call("rpc_get_block");
		stats.record_resource_read("ckb://docs/cell-model");
		stats.record_tool_call("prompt:create_script"); // prompt get path
		stats.record_request("list", "tools/list");

		let snapshot = stats.get_snapshot().unwrap();
		let feed: Vec<(&str, &str)> = snapshot
			.recent_requests
			.iter()
			.map(|e| (e.kind.as_str(), e.name.as_str()))
			.collect();

		// Newest-first; prompt is tagged `prompt` with the bare name (no prefix).
		assert_eq!(
			feed,
			vec![
				("list", "tools/list"),
				("prompt", "create_script"),
				("resource", "ckb://docs/cell-model"),
				("tool", "rpc_get_block"),
			],
			"feed must contain all four kinds, newest-first, correctly tagged"
		);

		// A list call has no per-item counter: it must NOT inflate Top Tools.
		assert!(
			!snapshot.tool_calls.iter().any(|e| e.name == "tools/list"),
			"list/discovery calls must not appear as tool-call counters"
		);
	}

	#[test]
	fn test_failed_requests_go_to_separate_error_feed_not_request_feed() {
		// Failures must populate the recent-FAILED-requests feed (with their
		// message), and must stay OUT of the success feed — the two feeds are
		// deliberately independent.
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("test_error_feed.redb");
		let stats = Stats::open(&db_path).unwrap();

		stats.record_resource_read("ckb://docs/ok");
		stats.record_error("tool", "rpc_get_block", "Connection refused");

		let snapshot = stats.get_snapshot().unwrap();

		// Success feed has only the successful read.
		assert_eq!(snapshot.recent_requests.len(), 1);
		assert_eq!(snapshot.recent_requests[0].kind, "resource");

		// Failure feed has the error, carrying kind, target, and message.
		assert_eq!(snapshot.recent_errors.len(), 1);
		let err = &snapshot.recent_errors[0];
		assert_eq!(err.kind, "tool");
		assert_eq!(err.name, "rpc_get_block");
		assert_eq!(err.detail, "Connection refused");

		// The human page renders the failures section with the message inline.
		let human = stats.format_human(None).unwrap();
		assert!(human.contains("Recent Failed Requests:"));
		assert!(human.contains("[tool] rpc_get_block - Connection refused ("));
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
		assert!(human.contains("1. [resource] ckb://docs/request-11 - "));
		assert!(human.contains("10. [resource] ckb://docs/request-2 - "));
		assert!(!human.contains("11. [resource] ckb://docs/request-1 - "));
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
		stats.record_error("tool", "get_block", "CKB RPC error: timeout");

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
		stats.record_error("tool", "rpc_get_block", "Connection refused");
		stats.record_error("tool", "rpc_get_block", "Connection refused");
		stats.record_error("tool", "rpc_get_block", "Connection refused");

		// Record different error for same source
		stats.record_error("tool", "rpc_get_block", "Timeout");

		// Record error for different source
		stats.record_error("tool", "dev_deploy", "Insufficient balance");

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
		stats.record_error("tool", "some_tool", &long_msg);

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
			stats.record_error("tool", "tool_a", "Some error");
			stats.record_error("tool", "tool_a", "Some error");
		}

		{
			let stats = Stats::open(&db_path).unwrap();
			stats.record_error("tool", "tool_a", "Some error");

			let snapshot = stats.get_snapshot().unwrap();
			assert_eq!(snapshot.total_errors, 3);
			assert_eq!(snapshot.error_summaries.len(), 1);
			assert_eq!(snapshot.error_summaries[0].count, 3);
		}
	}

	// --- Incompatibility detection + reset/fail policy ---------------------

	/// Write a redb database at `path` whose `tool_calls` table has the WRONG
	/// value type (`&str` instead of `u64`). The file opens fine, but our schema
	/// probe must reject it as incompatible — this simulates an on-disk layout
	/// from a different/older build.
	fn write_schema_incompatible_db(path: &std::path::Path) {
		const WRONG_TOOL_CALLS: TableDefinition<&str, &str> =
			TableDefinition::new("tool_calls");
		let db = Database::create(path).unwrap();
		let txn = db.begin_write().unwrap();
		{
			let mut t = txn.open_table(WRONG_TOOL_CALLS).unwrap();
			t.insert("get_block", "not-a-number").unwrap();
		}
		txn.commit().unwrap();
		// Drop the handle so the file lock is released before reopening.
		drop(db);
	}

	#[test]
	fn healthy_db_opens_under_both_policies_without_deletion() {
		// A normal DB must never be flagged as incompatible — guards against a
		// probe that is too aggressive and would silently wipe live telemetry.
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("healthy.redb");

		{
			let stats = Stats::open_with_policy(&db_path, OnIncompatible::Reset).unwrap();
			stats.record_tool_call("rpc_get_block");
		}
		// Reopen under Fail (the stricter policy): must succeed AND preserve data.
		let stats = Stats::open_with_policy(&db_path, OnIncompatible::Fail).unwrap();
		let snapshot = stats.get_snapshot().unwrap();
		assert_eq!(
			snapshot.total_tool_calls, 1,
			"healthy DB must open under Fail policy with data intact"
		);
	}

	#[test]
	fn schema_incompatible_db_is_reset_then_usable() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("schema_mismatch.redb");
		write_schema_incompatible_db(&db_path);

		// Reset policy: the incompatible file is replaced with a fresh, working DB.
		let stats = Stats::open_with_policy(&db_path, OnIncompatible::Reset).unwrap();
		stats.record_tool_call("rpc_get_block");
		let snapshot = stats.get_snapshot().unwrap();
		assert_eq!(
			snapshot.total_tool_calls, 1,
			"after reset the DB must be a clean, writable schema"
		);
	}

	#[test]
	fn schema_incompatible_db_fails_and_is_preserved_under_fail_policy() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("schema_mismatch_keep.redb");
		write_schema_incompatible_db(&db_path);
		let size_before = std::fs::metadata(&db_path).unwrap().len();

		// Fail policy: must error AND leave the file untouched for inspection.
		let result = Stats::open_with_policy(&db_path, OnIncompatible::Fail);
		assert!(
			result.is_err(),
			"Fail policy must refuse to open an incompatible DB"
		);
		assert!(
			db_path.exists(),
			"Fail policy must NOT delete the incompatible file"
		);
		assert_eq!(
			std::fs::metadata(&db_path).unwrap().len(),
			size_before,
			"the preserved file must be byte-for-byte unchanged"
		);
	}

	#[test]
	fn corrupt_file_is_reset_under_default_policy() {
		// A structurally garbled file (not a valid redb database) must be
		// detected at open time and reset under the default policy.
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("corrupt.redb");
		std::fs::write(&db_path, b"this is not a redb database, just garbage bytes").unwrap();

		let stats = Stats::open(&db_path).unwrap(); // default = Reset
		stats.record_tool_call("rpc_get_block");
		assert_eq!(stats.get_snapshot().unwrap().total_tool_calls, 1);
	}

	#[test]
	fn corrupt_file_is_preserved_under_fail_policy() {
		let dir = tempdir().unwrap();
		let db_path = dir.path().join("corrupt_keep.redb");
		let garbage = b"this is not a redb database, just garbage bytes";
		std::fs::write(&db_path, garbage).unwrap();

		let result = Stats::open_with_policy(&db_path, OnIncompatible::Fail);
		assert!(result.is_err(), "Fail policy must error on a corrupt file");
		assert!(db_path.exists(), "Fail policy must preserve the corrupt file");
		assert_eq!(
			std::fs::read(&db_path).unwrap(),
			garbage,
			"the preserved corrupt file must be unchanged"
		);
	}
}
