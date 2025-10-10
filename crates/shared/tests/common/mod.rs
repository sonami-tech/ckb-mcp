use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::sync::OnceLock;

/// Shared test data collected once during Phase 3 setup.
/// This data is gathered via direct CKB RPC calls (not through MCP).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SharedTestData {
	pub chain_type: String,
	pub genesis_hash: String,
	pub genesis_block: Value,
}

/// Global shared test data, initialized once in Phase 3.
static SHARED_DATA: OnceLock<SharedTestData> = OnceLock::new();

impl SharedTestData {
	/// Collect shared test data by querying CKB RPC directly.
	/// Returns the collected data without storing it.
	async fn collect() -> Result<SharedTestData, String> {
		let client = Client::new();
		let ckb_rpc_url = TestContext::get_ckb_rpc_url()?;

		// Get genesis block (block 0)
		let genesis_response = client
			.post(&ckb_rpc_url)
			.json(&json!({
				"jsonrpc": "2.0",
				"id": 1,
				"method": "get_block_by_number",
				"params": ["0x0"]
			}))
			.send()
			.await
			.map_err(|e| format!("Failed to fetch genesis block: {}", e))?;

		let genesis_body: Value = genesis_response
			.json()
			.await
			.map_err(|e| format!("Failed to parse genesis block response: {}", e))?;

		if let Some(error) = genesis_body.get("error") {
			return Err(format!("CKB RPC error fetching genesis: {}", error));
		}

		let genesis_block = genesis_body
			.get("result")
			.ok_or("Genesis block not found in response")?
			.clone();

		let genesis_hash = genesis_block["header"]["hash"]
			.as_str()
			.ok_or("Genesis hash not found")?
			.to_string();

		// Determine chain type from genesis hash
		let chain_type = match genesis_hash.as_str() {
			"0x92b197aa1fba0f63633922c61c92375c9c074a93e85963554f5499fe1450d0e5" => "mainnet",
			"0x10639e0895502b5688a6be8cf69460d76541bfa4821629d86d62ba0aae3f9606" => "testnet",
			_ => "devnet",
		}
		.to_string();

		Ok(SharedTestData {
			chain_type,
			genesis_hash,
			genesis_block,
		})
	}

	/// Initialize shared test data by querying CKB RPC directly.
	/// This should be called once in the Phase 3 test.
	#[allow(dead_code)]
	pub async fn initialize() -> Result<(), String> {
		let data = Self::collect().await?;
		SHARED_DATA
			.set(data)
			.map_err(|_| "SharedTestData already initialized".to_string())?;
		Ok(())
	}

	/// Get the shared test data. Returns None if not yet initialized.
	#[allow(dead_code)]
	pub fn get() -> Option<&'static SharedTestData> {
		SHARED_DATA.get()
	}

	/// Get the shared test data, initializing if not yet done.
	/// This allows tests to run in any order while still collecting data only once.
	/// MUST be called from within an async context (tokio test).
	#[allow(dead_code)]
	pub async fn get_or_init_async() -> &'static SharedTestData {
		if let Some(data) = SHARED_DATA.get() {
			return data;
		}

		// Need to initialize - collect data
		let data = Self::collect().await.expect("Failed to collect SharedTestData");

		// Try to store it, but if another thread beat us to it, that's fine
		let _ = SHARED_DATA.set(data);

		SHARED_DATA.get().expect("Data should be initialized")
	}
}

pub struct TestContext {
	pub client: Client,
	pub base_url: String,
}

impl TestContext {
	/// Get CKB RPC URL from CKB_RPC_URL environment variable.
	/// This should be set when running tests, matching the URL passed to the servers.
	pub fn get_ckb_rpc_url() -> Result<String, String> {
		env::var("CKB_RPC_URL").map_err(|_| {
			"CKB_RPC_URL environment variable not set. Please set it when running tests.".to_string()
		})
	}
}

impl TestContext {
	pub fn new(port: u16) -> Self {
		// Configure client with reasonable timeouts for tests.
		let client = Client::builder()
			.timeout(std::time::Duration::from_secs(30))
			.connect_timeout(std::time::Duration::from_secs(5))
			.build()
			.expect("Failed to build HTTP client");

		Self {
			client,
			base_url: format!("http://localhost:{}", port),
		}
	}

	/// Sanity check: verify server is running and healthy
	#[allow(dead_code)]
	pub async fn verify_server_running(&self) -> Result<(), String> {
		let health_url = format!("{}/health", self.base_url);

		match self.client.get(&health_url).send().await {
			Ok(resp) if resp.status().is_success() => Ok(()),
			Ok(resp) => Err(format!("Server unhealthy: status {}", resp.status())),
			Err(e) => Err(format!(
				"Server not responding at {}. Is it running? Error: {}",
				self.base_url, e
			)),
		}
	}

	/// Call MCP endpoint
	pub async fn mcp_call(&self, method: &str, params: Value) -> Result<Value, String> {
		let response = self
			.client
			.post(format!("{}/mcp", self.base_url))
			.json(&json!({
				"jsonrpc": "2.0",
				"id": 1,
				"method": method,
				"params": params
			}))
			.send()
			.await
			.map_err(|e| e.to_string())?;

		let body: Value = response.json().await.map_err(|e| e.to_string())?;

		if let Some(error) = body.get("error") {
			return Err(format!("MCP error: {}", error));
		}

		Ok(body["result"].clone())
	}

	/// Call an MCP tool
	#[allow(dead_code)]
	pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, String> {
		self.mcp_call("tools/call", json!({ "name": name, "arguments": arguments }))
			.await
	}

	/// Wait for a transaction to be confirmed by polling the CKB RPC directly.
	/// Returns the block number where the transaction was confirmed.
	#[allow(dead_code)]
	pub async fn wait_for_tx_confirmation(tx_hash: &str) -> Result<u64, String> {
		let client = Client::new();
		let ckb_rpc_url = Self::get_ckb_rpc_url()?;

		// Poll for up to 60 seconds
		for _ in 0..60 {
			let response = client
				.post(&ckb_rpc_url)
				.json(&json!({
					"jsonrpc": "2.0",
					"id": 1,
					"method": "get_transaction",
					"params": [tx_hash]
				}))
				.send()
				.await
				.map_err(|e| e.to_string())?;

			let body: Value = response.json().await.map_err(|e| e.to_string())?;

			if let Some(error) = body.get("error") {
				return Err(format!("RPC error: {}", error));
			}

			// Check if transaction is confirmed (has tx_status.status == "committed")
			if let Some(result) = body.get("result") {
				if let Some(tx_status) = result.get("tx_status") {
					if let Some(status) = tx_status.get("status").and_then(|s| s.as_str()) {
						if status == "committed" {
							// Extract block_number from the response
							if let Some(block_hash) = tx_status.get("block_hash").and_then(|h| h.as_str()) {
								// Get the block header to extract block number
								let block_response = client
									.post(&ckb_rpc_url)
									.json(&json!({
										"jsonrpc": "2.0",
										"id": 2,
										"method": "get_header",
										"params": [block_hash]
									}))
									.send()
									.await
									.map_err(|e| e.to_string())?;

								let block_body: Value = block_response.json().await.map_err(|e| e.to_string())?;

								if let Some(header) = block_body.get("result") {
									if let Some(number_hex) = header.get("number").and_then(|n| n.as_str()) {
										if let Ok(block_number) = u64::from_str_radix(number_hex.trim_start_matches("0x"), 16) {
											return Ok(block_number);
										}
									}
								}
							}
							return Err("Transaction confirmed but couldn't parse block number".to_string());
						}
					}
				}
			}

			tokio::time::sleep(std::time::Duration::from_secs(1)).await;
		}

		Err("Transaction confirmation timeout".to_string())
	}

	/// Wait for the indexer to catch up to at least the specified block number
	/// This ensures cells from confirmed transactions are available for collection
	#[allow(dead_code)]
	pub async fn wait_for_indexer_sync(target_block: u64) -> Result<(), String> {
		let client = Client::new();
		let ckb_rpc_url = Self::get_ckb_rpc_url()?;

		// Poll for up to 30 seconds
		for _ in 0..30 {
			let response = client
				.post(&ckb_rpc_url)
				.json(&json!({
					"jsonrpc": "2.0",
					"id": 1,
					"method": "get_indexer_tip",
					"params": []
				}))
				.send()
				.await
				.map_err(|e| e.to_string())?;

			let body: Value = response.json().await.map_err(|e| e.to_string())?;

			if let Some(error) = body.get("error") {
				return Err(format!("RPC error: {}", error));
			}

			if let Some(result) = body.get("result") {
				if let Some(block_num_hex) = result.get("block_number").and_then(|v| v.as_str()) {
					if let Ok(indexer_tip) = u64::from_str_radix(block_num_hex.trim_start_matches("0x"), 16) {
						if indexer_tip >= target_block {
							return Ok(());
						}
					}
				}
			}

			tokio::time::sleep(std::time::Duration::from_millis(500)).await;
		}

		Err(format!("Indexer failed to sync to block {}", target_block))
	}
}
