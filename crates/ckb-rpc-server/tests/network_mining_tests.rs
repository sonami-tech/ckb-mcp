use serde_json::json;
use test_common::{SharedTestData, TestContext};

const RPC_SERVER_PORT: u16 = 8001;

#[tokio::test]
async fn test_local_node_info() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "local_node_info", "arguments": {}}))
		.await
		.expect("local_node_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();

	// Parse JSON to validate structure
	let node_info: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify ALL required fields exist (not just one)
	assert!(node_info.get("version").is_some(), "Should contain version field");
	assert!(node_info.get("node_id").is_some(), "Should contain node_id field");
	assert!(node_info.get("addresses").is_some(), "Should contain addresses field");

	// Validate field types and formats
	let version = node_info["version"].as_str()
		.expect("version should be a string");
	assert!(!version.is_empty(), "version should not be empty");

	let node_id = node_info["node_id"].as_str()
		.expect("node_id should be a string");
	assert!(!node_id.is_empty(), "node_id should not be empty");

	let _addresses = node_info["addresses"].as_array()
		.expect("addresses should be an array");
	// Note: addresses could be empty if node has no peers, so we just check it's an array

	// Validate connections is a hex number string if present
	if let Some(connections) = node_info.get("connections") {
		let conn_str = connections.as_str().expect("connections should be a string");
		assert!(conn_str.starts_with("0x"), "connections should be hex format");
	}
}

// General Error Cases
#[tokio::test]
async fn test_get_peers() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_peers", "arguments": {}}))
		.await
		.expect("get_peers should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let peers: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Should be an array
	let peers_array = peers.as_array().expect("Response should be an array");

	// If there are peers, verify structure
	if !peers_array.is_empty() {
		let peer = &peers_array[0];

		// Verify key peer fields
		assert!(peer.get("node_id").is_some(), "Peer should have 'node_id' field");
		assert!(peer.get("addresses").is_some(), "Peer should have 'addresses' field");
		assert!(peer.get("is_outbound").is_some(), "Peer should have 'is_outbound' field");
		assert!(peer.get("connected_duration").is_some(), "Peer should have 'connected_duration' field");
		assert!(peer.get("protocols").is_some(), "Peer should have 'protocols' field");
		assert!(peer.get("version").is_some(), "Peer should have 'version' field");

		// Verify node_id is a string
		peer["node_id"].as_str().expect("node_id should be a string");

		// Verify is_outbound is boolean
		peer["is_outbound"].as_bool().expect("is_outbound should be a boolean");

		// Verify addresses is an array
		let addresses = peer["addresses"].as_array().expect("addresses should be an array");
		if !addresses.is_empty() {
			assert!(addresses[0].get("address").is_some(), "Address should have 'address' field");
			assert!(addresses[0].get("score").is_some(), "Address should have 'score' field");
		}

		// Verify protocols is an array
		let protocols = peer["protocols"].as_array().expect("protocols should be an array");
		if !protocols.is_empty() {
			assert!(protocols[0].get("id").is_some(), "Protocol should have 'id' field");
			assert!(protocols[0].get("version").is_some(), "Protocol should have 'version' field");
		}
	}
}

#[tokio::test]
async fn test_sync_state() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "sync_state", "arguments": {}}))
		.await
		.expect("sync_state should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let sync_state: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify key sync state fields
	assert!(sync_state.get("ibd").is_some(), "Response should have 'ibd' field");
	assert!(sync_state.get("tip_hash").is_some(), "Response should have 'tip_hash' field");
	assert!(sync_state.get("tip_number").is_some(), "Response should have 'tip_number' field");
	assert!(sync_state.get("best_known_block_number").is_some(), "Response should have 'best_known_block_number' field");
	assert!(sync_state.get("best_known_block_timestamp").is_some(), "Response should have 'best_known_block_timestamp' field");
	assert!(sync_state.get("orphan_blocks_count").is_some(), "Response should have 'orphan_blocks_count' field");
	assert!(sync_state.get("inflight_blocks_count").is_some(), "Response should have 'inflight_blocks_count' field");
	assert!(sync_state.get("fast_time").is_some(), "Response should have 'fast_time' field");
	assert!(sync_state.get("normal_time").is_some(), "Response should have 'normal_time' field");
	assert!(sync_state.get("low_time").is_some(), "Response should have 'low_time' field");

	// Verify ibd is boolean
	sync_state["ibd"].as_bool().expect("ibd should be a boolean");

	// Verify tip_hash format
	let tip_hash = sync_state["tip_hash"].as_str().expect("tip_hash should be a string");
	assert!(tip_hash.starts_with("0x"), "tip_hash should be in hex format");
	assert_eq!(tip_hash.len(), 66, "tip_hash should be 66 characters");

	// Verify numeric fields are in hex format
	let tip_number = sync_state["tip_number"].as_str().expect("tip_number should be a string");
	assert!(tip_number.starts_with("0x"), "tip_number should be in hex format");
}

#[tokio::test]
async fn test_get_consensus() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_consensus", "arguments": {}}))
		.await
		.expect("get_consensus should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let consensus: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify key consensus parameters exist
	assert!(consensus.get("id").is_some(), "Response should have 'id' field");
	assert!(consensus.get("genesis_hash").is_some(), "Response should have 'genesis_hash' field");
	assert!(consensus.get("dao_type_hash").is_some(), "Response should have 'dao_type_hash' field");
	assert!(consensus.get("epoch_duration_target").is_some(), "Response should have 'epoch_duration_target' field");
	assert!(consensus.get("hardfork_features").is_some(), "Response should have 'hardfork_features' field");

	// Verify genesis_hash format
	let genesis_hash = consensus["genesis_hash"].as_str().expect("genesis_hash should be a string");
	assert!(genesis_hash.starts_with("0x"), "genesis_hash should be in hex format");
	assert_eq!(genesis_hash.len(), 66, "genesis_hash should be 66 characters (0x + 64 hex digits)");

	// Verify id field (chain identifier)
	let id = consensus["id"].as_str().expect("id should be a string");
	assert!(!id.is_empty(), "id should not be empty");

	// Verify hardfork_features is an array
	assert!(consensus["hardfork_features"].is_array(), "hardfork_features should be an array");
}

#[tokio::test]
async fn test_get_deployments_info() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "get_deployments_info", "arguments": {}}))
		.await
		.expect("get_deployments_info should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let deployments_info: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify key deployment info fields
	assert!(deployments_info.get("hash").is_some(), "Response should have 'hash' field");
	assert!(deployments_info.get("epoch").is_some(), "Response should have 'epoch' field");
	assert!(deployments_info.get("deployments").is_some(), "Response should have 'deployments' field");

	// Verify hash format
	let hash = deployments_info["hash"].as_str().expect("hash should be a string");
	assert!(hash.starts_with("0x"), "hash should be in hex format");
	assert_eq!(hash.len(), 66, "hash should be 66 characters");

	// Verify epoch format
	let epoch = deployments_info["epoch"].as_str().expect("epoch should be a string");
	assert!(epoch.starts_with("0x"), "epoch should be in hex format");

	// Verify deployments is an object
	let deployments = deployments_info["deployments"].as_object().expect("deployments should be an object");

	// If there are deployments, verify structure
	for (deployment_name, deployment_info) in deployments {
		// Deployment should have state and bit fields at minimum
		assert!(deployment_info.get("state").is_some(), "Deployment '{}' should have 'state' field", deployment_name);
		assert!(deployment_info.get("bit").is_some(), "Deployment '{}' should have 'bit' field", deployment_name);

		// Verify state is a string
		deployment_info["state"].as_str().expect(&format!("Deployment '{}' state should be a string", deployment_name));

		// Verify bit is a number (in hex format)
		deployment_info["bit"].as_u64().or_else(|| deployment_info["bit"].as_str().and_then(|s| u64::from_str_radix(&s[2..], 16).ok()))
			.expect(&format!("Deployment '{}' bit should be a number", deployment_name));
	}
}

// TODO: Implement success case testing for estimate_cycles
//
// This test is commented out because ckb-rpc-server has no transaction signing capability.
// Testing estimate_cycles success case requires:
// 1. Finding live (unspent) cells
// 2. Building a valid transaction structure
// 3. Signing the transaction with a valid private key (ckb-rpc-server doesn't have this)
//
// Options to implement this test:
// - Move to ckb-tools-server tests (has private key and signing infrastructure)
// - Add a test-only private key to ckb-rpc-server test harness
// - Use unsigned transaction that doesn't require signature validation
//
// Error cases remain fully tested via:
// - test_estimate_cycles_missing_tx (missing parameter)
// - test_estimate_cycles_invalid_tx (malformed transaction)
//
// #[tokio::test]
// async fn test_estimate_cycles() {
// 	// Implementation would go here
// }

#[tokio::test]
async fn test_estimate_cycles_missing_tx() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "estimate_cycles", "arguments": {}}))
		.await;

	assert!(result.is_err(), "Should fail when tx parameter is missing");
	let error_msg = result.unwrap_err();
	assert!(error_msg.contains("Missing tx"), "Error should mention missing tx parameter");
}

#[tokio::test]
async fn test_estimate_cycles_invalid_tx() {
	let ctx = TestContext::new(RPC_SERVER_PORT);
	let shared_data = SharedTestData::get_or_init_async().await;

	// Use genesis cellbase which has unresolvable inputs (null outpoint)
	let genesis_cellbase = &shared_data.genesis_block["transactions"][0];

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "estimate_cycles",
			"arguments": {
				"tx": genesis_cellbase
			}
		}))
		.await;

	// Should fail because genesis cellbase references null outpoint
	assert!(result.is_err(), "Should fail for transaction with unresolvable inputs");
	let error_msg = result.unwrap_err();
	// Error can be either TransactionFailedToResolve or just contain "error"
	assert!(error_msg.to_lowercase().contains("error") || error_msg.contains("Failed"),
		"Error should indicate failure, got: {}", error_msg);
}

// Pool Methods

#[tokio::test]
async fn test_estimate_fee_rate() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({"name": "estimate_fee_rate", "arguments": {}}))
		.await
		.expect("estimate_fee_rate should succeed");

	let content = result["content"][0]["text"].as_str().unwrap();
	let fee_rate: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Should be a hex string representing shannons per KB
	let fee_rate_str = fee_rate.as_str().expect("fee_rate should be a string");
	assert!(fee_rate_str.starts_with("0x"), "fee_rate should be in hex format");

	// Parse as u64 to verify it's a valid number
	let fee_value = u64::from_str_radix(&fee_rate_str[2..], 16)
		.expect("fee_rate should be valid hex number");
	assert!(fee_value > 0, "fee_rate should be greater than 0");
}

#[tokio::test]
async fn test_estimate_fee_rate_with_params() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "estimate_fee_rate",
			"arguments": {
				"estimate_mode": "no_priority",
				"enable_fallback": true
			}
		}))
		.await
		.expect("estimate_fee_rate should succeed with params");

	let content = result["content"][0]["text"].as_str().unwrap();
	let fee_rate: serde_json::Value = serde_json::from_str(content)
		.expect("Response should be valid JSON");

	// Verify it's a hex string
	let fee_rate_str = fee_rate.as_str().expect("fee_rate should be a string");
	assert!(fee_rate_str.starts_with("0x"), "fee_rate should be in hex format");
}

#[tokio::test]
async fn test_calculate_dao_maximum_withdraw_missing_params() {
	let ctx = TestContext::new(RPC_SERVER_PORT);

	// Test missing out_point
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "calculate_dao_maximum_withdraw",
			"arguments": {
				"kind": "0xa5f5c85987a15de25661e5a214f2c1449cd803f071acc7999820f25246471f40"
			}
		}))
		.await;

	assert!(result.is_err(), "Should fail when out_point is missing");

	// Test missing kind
	let result = ctx
		.mcp_call("tools/call", json!({
			"name": "calculate_dao_maximum_withdraw",
			"arguments": {
				"out_point": {
					"tx_hash": "0xa4037a893eb48e18ed4ef61034ce26eba9c585f15c9cee102ae58505565eccc3",
					"index": "0x0"
				}
			}
		}))
		.await;

	assert!(result.is_err(), "Should fail when kind is missing");
}

// Note: Removed mining-related tests (test_submit_block_missing_work_id, test_submit_block_missing_block)
// These require the Miner RPC module to be enabled on the CKB node, which is typically disabled
// on public devnet/testnet nodes. Mining operations are not relevant for AI-assisted smart contract
// development workflows.
