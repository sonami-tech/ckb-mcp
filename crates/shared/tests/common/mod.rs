use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct TestContext {
	pub client: Client,
	pub base_url: String,
}

impl TestContext {
	/// Get CKB RPC URL from CKB_RPC_URL environment variable.
	/// This should be set when running tests, matching the URL passed to the servers.
	fn get_ckb_rpc_url() -> Result<String, String> {
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
