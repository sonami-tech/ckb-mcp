use reqwest::Client;
use serde_json::{json, Value};

pub struct TestContext {
	pub client: Client,
	pub base_url: String,
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

	/// Wait for a transaction to be confirmed by polling via MCP tools/call
	/// This method requires the RPC server context (port 8001) to query transaction status.
	/// Returns the block number where the transaction was confirmed.
	#[allow(dead_code)]
	pub async fn wait_for_tx_confirmation(
		rpc_server_ctx: &TestContext,
		tx_hash: &str,
	) -> Result<u64, String> {
		// Poll for up to 60 seconds
		for _ in 0..60 {
			let result = rpc_server_ctx
				.call_tool("get_transaction", json!({ "tx_hash": tx_hash }))
				.await?;

			// Check if transaction is confirmed (has tx_status.status == "committed")
			if let Some(content) = result.get("content") {
				if let Some(content_array) = content.as_array() {
					if let Some(first) = content_array.first() {
						if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
							if text.contains("\"status\": \"committed\"") {
								// Extract block_number from the response
								if let Some(block_num_start) = text.find("\"block_number\": \"0x") {
									let hex_start = block_num_start + 19;
									if let Some(hex_end) = text[hex_start..].find('"') {
										let block_hex = &text[hex_start..hex_start + hex_end];
										if let Ok(block_number) = u64::from_str_radix(block_hex, 16) {
											return Ok(block_number);
										}
									}
								}
								// Fallback if we can't parse block number
								return Err("Transaction confirmed but couldn't parse block number".to_string());
							}
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
	pub async fn wait_for_indexer_sync(
		rpc_server_ctx: &TestContext,
		target_block: u64,
	) -> Result<(), String> {
		// Poll for up to 30 seconds
		for _ in 0..30 {
			let result = rpc_server_ctx
				.call_tool("get_indexer_tip", json!({}))
				.await?;

			if let Some(content) = result.get("content") {
				if let Some(content_array) = content.as_array() {
					if let Some(first) = content_array.first() {
						if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
							// Parse the JSON response to get block_number
							if let Ok(tip_info) = serde_json::from_str::<Value>(text) {
								if let Some(block_num_hex) = tip_info.get("block_number").and_then(|v| v.as_str()) {
									if let Ok(indexer_tip) = u64::from_str_radix(block_num_hex.trim_start_matches("0x"), 16) {
										if indexer_tip >= target_block {
											return Ok(());
										}
									}
								}
							}
						}
					}
				}
			}

			tokio::time::sleep(std::time::Duration::from_millis(500)).await;
		}

		Err(format!("Indexer failed to sync to block {}", target_block))
	}
}
