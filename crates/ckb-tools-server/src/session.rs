use fs2::FileExt;
use serde::{Deserialize, Serialize};
use shared::error::{CkbMcpError, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, error};
use uuid::Uuid;

// Constants
pub const MAX_FILE_SIZE: usize = 350 * 1024; // 350KB
pub const MAX_CHUNK_SIZE: usize = 50 * 1024; // 50KB
#[allow(dead_code)]
pub const RECOMMENDED_CHUNK_SIZE: usize = 10 * 1024; // 10KB (documented in API, not enforced)
const SESSION_DIR: &str = "/tmp/ckb_sessions";
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const CANCEL_LOCK_TIMEOUT: Duration = Duration::from_secs(2);

/// Session metadata stored in metadata.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
	pub session_key: String,
	pub state: SessionState,
	pub expected_size: usize,
	pub total_bytes: usize,
	pub error_message: Option<String>,
	pub sha256_hash: Option<String>,
	pub blake2b_hash: Option<String>,
	pub ckb_hash: Option<String>,
}

/// Session states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
	Receiving,
	Finalized,
	Deployed,
	Error,
}

impl SessionMetadata {
	/// Create new session metadata in receiving state.
	pub fn new(session_key: String, expected_size: usize) -> Self {
		Self {
			session_key,
			state: SessionState::Receiving,
			expected_size,
			total_bytes: 0,
			error_message: None,
			sha256_hash: None,
			blake2b_hash: None,
			ckb_hash: None,
		}
	}
}

/// Session file paths.
pub struct SessionPaths {
	pub session_dir: PathBuf,
	pub data_file: PathBuf,
	pub metadata_file: PathBuf,
	pub lock_file: PathBuf,
}

impl SessionPaths {
	/// Get paths for a session key.
	pub fn new(session_key: &str) -> Self {
		let session_dir = PathBuf::from(SESSION_DIR).join(session_key);
		let data_file = session_dir.join("data.bin");
		let metadata_file = session_dir.join("metadata.json");
		let lock_file = session_dir.join("session.lock");

		Self {
			session_dir,
			data_file,
			metadata_file,
			lock_file,
		}
	}
}

/// Session lock guard that automatically releases lock on drop.
pub struct SessionLock {
	_file: File,
}

impl SessionLock {
	/// Acquire exclusive lock on session with timeout.
	pub fn acquire(lock_path: &Path, timeout: Duration) -> Result<Self> {
		// Create lock file if it doesn't exist.
		let file = OpenOptions::new()
			.create(true)
			.read(true)
			.write(true)
			.open(lock_path)
			.map_err(|e| {
				CkbMcpError::Internal(format!("Failed to open lock file: {}", e))
			})?;

		// Try to acquire lock with timeout.
		let start = std::time::Instant::now();
		loop {
			match file.try_lock_exclusive() {
				Ok(_) => {
					return Ok(Self { _file: file });
				}
				Err(_) if start.elapsed() < timeout => {
					std::thread::sleep(Duration::from_millis(100));
					continue;
				}
				Err(e) => {
					return Err(CkbMcpError::Internal(format!(
						"Failed to acquire lock after {:?}: {}",
						timeout, e
					)));
				}
			}
		}
	}
}

impl Drop for SessionLock {
	fn drop(&mut self) {
		// Lock is automatically released when file is closed.
	}
}

/// Session manager for chunked uploads.
pub struct SessionManager;

impl SessionManager {
	/// Create new session with expected size.
	pub fn create_session(expected_size: usize) -> Result<SessionMetadata> {
		// Validate expected size.
		if expected_size > MAX_FILE_SIZE {
			return Err(CkbMcpError::InvalidParameter(format!(
				"Expected size {} exceeds maximum file size {} (350KB)",
				expected_size, MAX_FILE_SIZE
			)));
		}

		// Generate session key.
		let session_key = Uuid::new_v4().to_string();
		let paths = SessionPaths::new(&session_key);

		// Create session directory.
		fs::create_dir_all(&paths.session_dir).map_err(|e| {
			CkbMcpError::Internal(format!("Failed to create session directory: {}", e))
		})?;

		// Create metadata.
		let metadata = SessionMetadata::new(session_key, expected_size);

		// Acquire lock and write metadata.
		let _lock = SessionLock::acquire(&paths.lock_file, LOCK_TIMEOUT)?;
		Self::write_metadata(&paths.metadata_file, &metadata)?;

		debug!(
			"Created session {} with expected size {}",
			metadata.session_key, expected_size
		);

		Ok(metadata)
	}

	/// Read session metadata with lock.
	pub fn read_metadata_locked(session_key: &str) -> Result<(SessionMetadata, SessionLock)> {
		let paths = SessionPaths::new(session_key);

		if !paths.session_dir.exists() {
			return Err(CkbMcpError::NotFound(format!(
				"Session {} not found",
				session_key
			)));
		}

		let lock = SessionLock::acquire(&paths.lock_file, LOCK_TIMEOUT)?;
		let metadata = Self::read_metadata(&paths.metadata_file)?;

		Ok((metadata, lock))
	}

	/// Read session metadata without lock (for status queries).
	pub fn read_metadata_unlocked(session_key: &str) -> Result<SessionMetadata> {
		let paths = SessionPaths::new(session_key);

		if !paths.session_dir.exists() {
			return Err(CkbMcpError::NotFound(format!(
				"Session {} not found",
				session_key
			)));
		}

		Self::read_metadata(&paths.metadata_file)
	}

	/// Write metadata to file.
	/// LOCKING: Caller must hold the session lock.
	fn write_metadata(path: &Path, metadata: &SessionMetadata) -> Result<()> {
		let json = serde_json::to_string_pretty(metadata)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to serialize metadata: {}", e)))?;

		fs::write(path, json)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to write metadata: {}", e)))?;

		Ok(())
	}

	/// Read metadata from file.
	fn read_metadata(path: &Path) -> Result<SessionMetadata> {
		let json = fs::read_to_string(path)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to read metadata: {}", e)))?;

		let metadata: SessionMetadata = serde_json::from_str(&json)
			.map_err(|e| CkbMcpError::Internal(format!("Failed to parse metadata: {}", e)))?;

		Ok(metadata)
	}

	/// Update metadata in session.
	/// LOCKING: Caller must hold the session lock.
	pub fn update_metadata(session_key: &str, metadata: &SessionMetadata) -> Result<()> {
		let paths = SessionPaths::new(session_key);
		Self::write_metadata(&paths.metadata_file, metadata)
	}

	/// Set session to error state.
	/// LOCKING: Caller must hold the session lock.
	pub fn set_error_state(session_key: &str, error_msg: &str) -> Result<()> {
		let paths = SessionPaths::new(session_key);

		// Try to read existing metadata.
		let mut metadata = match Self::read_metadata(&paths.metadata_file) {
			Ok(m) => m,
			Err(_) => {
				// If we can't read metadata, create minimal error metadata.
				SessionMetadata {
					session_key: session_key.to_string(),
					state: SessionState::Error,
					expected_size: 0,
					total_bytes: 0,
					error_message: Some(error_msg.to_string()),
					sha256_hash: None,
					blake2b_hash: None,
					ckb_hash: None,
				}
			}
		};

		metadata.state = SessionState::Error;
		metadata.error_message = Some(error_msg.to_string());

		Self::write_metadata(&paths.metadata_file, &metadata)?;
		error!("Session {} entered error state: {}", session_key, error_msg);

		Ok(())
	}

	/// Append data to session.
	pub fn append_data(session_key: &str, chunk_data: &[u8]) -> Result<SessionMetadata> {
		let paths = SessionPaths::new(session_key);

		// Acquire lock.
		let _lock = SessionLock::acquire(&paths.lock_file, LOCK_TIMEOUT)?;

		// Read metadata.
		let mut metadata = Self::read_metadata(&paths.metadata_file)?;

		// Check state.
		if metadata.state != SessionState::Receiving {
			return Err(CkbMcpError::InvalidParameter(format!(
				"Cannot append to session in {} state. Current state must be receiving.",
				serde_json::to_string(&metadata.state).unwrap_or_default()
			)));
		}

		// Check chunk size.
		if chunk_data.len() > MAX_CHUNK_SIZE {
			let error_msg = format!(
				"Chunk size {} exceeds maximum chunk size {} (50KB)",
				chunk_data.len(),
				MAX_CHUNK_SIZE
			);
			Self::set_error_state(session_key, &error_msg)?;
			return Err(CkbMcpError::InvalidParameter(error_msg));
		}

		// Check if would exceed expected size.
		let new_total = metadata.total_bytes + chunk_data.len();
		if new_total > metadata.expected_size {
			let error_msg = format!(
				"Chunk would exceed expected size ({} bytes). Attempted to write {} bytes with {} already received.",
				metadata.expected_size,
				new_total,
				metadata.total_bytes
			);
			Self::set_error_state(session_key, &error_msg)?;
			return Err(CkbMcpError::InvalidParameter(error_msg));
		}

		// Append to data file.
		let mut file = OpenOptions::new()
			.create(true)
			.append(true)
			.open(&paths.data_file)
			.map_err(|e| {
				let error_msg = format!("Failed to open data file: {}", e);
				let _ = Self::set_error_state(session_key, &error_msg);
				CkbMcpError::Internal(error_msg)
			})?;

		file.write_all(chunk_data).map_err(|e| {
			let error_msg = format!("Failed to write chunk data: {}", e);
			let _ = Self::set_error_state(session_key, &error_msg);
			CkbMcpError::Internal(error_msg)
		})?;

		// Update metadata.
		metadata.total_bytes = new_total;
		Self::write_metadata(&paths.metadata_file, &metadata).map_err(|e| {
			let error_msg = format!("Failed to update metadata after append: {}", e);
			let _ = Self::set_error_state(session_key, &error_msg);
			e
		})?;

		debug!(
			"Appended {} bytes to session {}, total: {}/{}",
			chunk_data.len(),
			session_key,
			metadata.total_bytes,
			metadata.expected_size
		);

		Ok(metadata)
	}

	/// Read all data from session.
	/// LOCKING: Caller should hold the session lock to ensure consistency.
	pub fn read_data(session_key: &str) -> Result<Vec<u8>> {
		let paths = SessionPaths::new(session_key);

		let mut file = File::open(&paths.data_file).map_err(|e| {
			CkbMcpError::Internal(format!("Failed to open data file: {}", e))
		})?;

		let mut data = Vec::new();
		file.read_to_end(&mut data).map_err(|e| {
			CkbMcpError::Internal(format!("Failed to read data file: {}", e))
		})?;

		Ok(data)
	}

	/// Delete session directory and all files.
	pub fn delete_session(session_key: &str) -> Result<()> {
		let paths = SessionPaths::new(session_key);

		if !paths.session_dir.exists() {
			return Err(CkbMcpError::NotFound(format!(
				"Session {} not found",
				session_key
			)));
		}

		// Try to acquire lock with shorter timeout for cancel.
		let _lock = match SessionLock::acquire(&paths.lock_file, CANCEL_LOCK_TIMEOUT) {
			Ok(lock) => Some(lock),
			Err(_) => {
				// If we can't get lock, force delete anyway (best effort).
				debug!("Could not acquire lock for cancel, forcing delete");
				None
			}
		};

		// Delete entire session directory.
		fs::remove_dir_all(&paths.session_dir).map_err(|e| {
			CkbMcpError::Internal(format!("Failed to delete session directory: {}", e))
		})?;

		debug!("Deleted session {}", session_key);

		Ok(())
	}
}
