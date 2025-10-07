use reqwest::Client;
use serde_json::{json, Value};

pub struct TestContext {
	pub client: Client,
	pub base_url: String,
}

impl TestContext {
	pub fn new(port: u16) -> Self {
		Self {
			client: Client::new(),
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
}
