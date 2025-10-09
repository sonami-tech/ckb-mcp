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
	pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, String> {
		self.mcp_call("tools/call", json!({ "name": name, "arguments": arguments }))
			.await
	}

	/// Wait for a transaction to be confirmed by polling via MCP tools/call
	/// This method requires the RPC server context (port 8001) to query transaction status.
	pub async fn wait_for_tx_confirmation(
		rpc_server_ctx: &TestContext,
		tx_hash: &str,
	) -> Result<(), String> {
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
								return Ok(());
							}
						}
					}
				}
			}

			tokio::time::sleep(std::time::Duration::from_secs(1)).await;
		}

		Err("Transaction confirmation timeout".to_string())
	}
}
