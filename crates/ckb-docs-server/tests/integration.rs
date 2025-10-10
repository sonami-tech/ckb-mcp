use serde_json::json;

#[path = "../../shared/tests/common/mod.rs"]
mod common;

use common::{SharedTestData, TestContext};

const DOCS_SERVER_PORT: u16 = 8002;

/// Phase 1: Verify MCP server is running
#[tokio::test]
async fn test_00_server_running() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	ctx.verify_server_running()
		.await
		.expect("ckb-docs-server must be running on port 8002. Start with: cargo run --bin ckb-docs-server");
}

/// Phase 2: Verify CKB RPC is available (direct connection, not through MCP)
#[tokio::test]
async fn test_01_ckb_rpc_available() {
	use reqwest::Client;

	let ckb_rpc_url = TestContext::get_ckb_rpc_url()
		.expect("CKB_RPC_URL must be set");

	let client = Client::new();
	let response = client
		.post(&ckb_rpc_url)
		.json(&json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": "get_tip_block_number",
			"params": []
		}))
		.send()
		.await
		.expect("CKB RPC should be accessible");

	let body: serde_json::Value = response.json().await.expect("Should parse JSON response");

	assert!(body.get("error").is_none(), "CKB RPC should not return error");
	assert!(body.get("result").is_some(), "CKB RPC should return result");
}

/// Phase 3: Collect shared test data from CKB RPC (not through MCP)
#[tokio::test]
async fn test_02_collect_shared_data() {
	SharedTestData::initialize()
		.await
		.expect("Should successfully collect shared test data from CKB RPC");

	let data = SharedTestData::get().expect("Shared data should be initialized");

	// Verify data was collected correctly
	assert!(!data.chain_type.is_empty(), "Chain type should not be empty");
	assert!(data.genesis_hash.starts_with("0x"), "Genesis hash should be hex format");
	assert!(data.genesis_block.get("header").is_some(), "Genesis block should have header");
}

#[tokio::test]
async fn test_mcp_initialize() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call(
			"initialize",
			json!({
				"protocolVersion": "2024-11-05",
				"capabilities": {},
				"clientInfo": {
					"name": "test-client",
					"version": "1.0.0"
				}
			}),
		)
		.await
		.expect("initialize should succeed");

	assert_eq!(result["protocolVersion"], "2024-11-05");
	assert!(result["serverInfo"]["name"]
		.as_str()
		.unwrap()
		.contains("ckb-docs"));
	assert!(result["capabilities"]["resources"].is_object());
}

#[tokio::test]
async fn test_resources_list() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/list", json!({}))
		.await
		.expect("resources/list should succeed");

	assert!(result["resources"].is_array());
	let resources = result["resources"].as_array().unwrap();
	assert!(!resources.is_empty(), "Should have documentation resources");
}

#[tokio::test]
async fn test_resources_list_all_have_descriptions() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/list", json!({}))
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	for resource in resources {
		let description = resource["description"].as_str().expect("Should have description");
		assert!(!description.is_empty(), "Description should not be empty");
		assert!(description.len() <= 1024, "Description should be under 1024 characters");
	}
}

#[tokio::test]
async fn test_resources_list_all_use_correct_uri_scheme() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/list", json!({}))
		.await
		.expect("resources/list should succeed");

	let resources = result["resources"].as_array().unwrap();
	for resource in resources {
		let uri = resource["uri"].as_str().expect("Should have URI");
		assert!(uri.starts_with("ckb-dev-context://"), "URI should use ckb-dev-context:// scheme");
	}
}

// Error Cases - 8 tests
#[tokio::test]
async fn test_read_nonexistent_resource() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({"uri": "ckb-dev-context://nonexistent/resource"}))
		.await;

	assert!(result.is_err(), "Should fail for nonexistent resource");
}

#[tokio::test]
async fn test_read_malformed_uri_no_scheme() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({"uri": "concepts/cell-model"}))
		.await;

	assert!(result.is_err(), "Should fail for URI without scheme");
}

#[tokio::test]
async fn test_read_malformed_uri_wrong_scheme() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({"uri": "http://concepts/cell-model"}))
		.await;

	assert!(result.is_err(), "Should fail for wrong URI scheme");
}

#[tokio::test]
async fn test_read_empty_uri() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({"uri": ""}))
		.await;

	assert!(result.is_err(), "Should fail for empty URI");
}

#[tokio::test]
async fn test_read_uri_trailing_slash() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({"uri": "ckb-dev-context://concepts/cell-model/"}))
		.await;

	assert!(result.is_err(), "Should fail for URI with trailing slash");
}

#[tokio::test]
async fn test_read_uri_double_slash() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({"uri": "ckb-dev-context://concepts//cell-model"}))
		.await;

	assert!(result.is_err(), "Should fail for URI with double slash");
}

#[tokio::test]
async fn test_resources_read_missing_params() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({}))
		.await;

	assert!(result.is_err(), "Should fail when URI parameter is missing");
}

#[tokio::test]
async fn test_resources_read_null_uri() {
	let ctx = TestContext::new(DOCS_SERVER_PORT);

	let result = ctx
		.mcp_call("resources/read", json!({"uri": null}))
		.await;

	assert!(result.is_err(), "Should fail for null URI");
}

// All 84 Resources Comprehensive Validation - one test per resource
macro_rules! test_resource {
	($name:ident, $uri:expr) => {
		#[tokio::test]
		async fn $name() {
			let ctx = TestContext::new(DOCS_SERVER_PORT);
			let result = ctx
				.mcp_call("resources/read", json!({"uri": $uri}))
				.await
				.expect(&format!("Should read {}", $uri));
			let content = result["contents"][0]["text"].as_str().unwrap();
			assert!(!content.is_empty(), "Content should not be empty for {}", $uri);
		}
	};
}

test_resource!(test_resource_ai_quick_reference, "ckb-dev-context://ai-quick-reference");
test_resource!(test_resource_ccc_api_patterns, "ckb-dev-context://api-reference/ccc-api-patterns");
test_resource!(test_resource_ckb_rust_sdk_practical_examples, "ckb-dev-context://api-reference/ckb-rust-sdk-practical-examples");
test_resource!(test_resource_cota_sdk_examples, "ckb-dev-context://api-reference/cota-sdk-examples");
test_resource!(test_resource_ickb_sdk_examples, "ckb-dev-context://api-reference/ickb-sdk-examples");
test_resource!(test_resource_molecule_api_examples, "ckb-dev-context://api-reference/molecule-api-examples");
test_resource!(test_resource_omnilock_api_examples, "ckb-dev-context://api-reference/omnilock-api-examples");
test_resource!(test_resource_sdk_examples_and_patterns, "ckb-dev-context://api-reference/sdk-examples-and-patterns");
test_resource!(test_resource_spore_sdk_examples, "ckb-dev-context://api-reference/spore-sdk-examples");
test_resource!(test_resource_syscalls_quick_ref_full, "ckb-dev-context://api-reference/syscalls-quick-ref");
test_resource!(test_resource_well_known_hashes, "ckb-dev-context://api-reference/well-known-hashes");
test_resource!(test_resource_ccc_sdk_cross_chain, "ckb-dev-context://api-reference/ccc-sdk-cross-chain");
test_resource!(test_resource_ccc_sdk_ssri, "ckb-dev-context://api-reference/ccc-sdk-ssri");
test_resource!(test_resource_omnilock_ethereum_example, "ckb-dev-context://api-reference/omnilock-ethereum-example");
test_resource!(test_resource_xudt_minting_example, "ckb-dev-context://api-reference/xudt-minting-example");

test_resource!(test_resource_advanced_cell_concepts, "ckb-dev-context://concepts/advanced-cell-concepts");
test_resource!(test_resource_cell_model_full, "ckb-dev-context://concepts/cell-model");
test_resource!(test_resource_ckb_syscalls_and_sources, "ckb-dev-context://concepts/ckb-syscalls-and-sources");
test_resource!(test_resource_ckb_network_history, "ckb-dev-context://concepts/ckb-network-history");
test_resource!(test_resource_molecule_serialization, "ckb-dev-context://concepts/molecule-serialization");
test_resource!(test_resource_script_groups_and_execution, "ckb-dev-context://concepts/script-groups-and-execution");
test_resource!(test_resource_transaction_structure_full, "ckb-dev-context://concepts/transaction-structure");
test_resource!(test_resource_header_dependencies_and_time_access, "ckb-dev-context://concepts/header-dependencies-and-time-access");
test_resource!(test_resource_lock_value_relationships, "ckb-dev-context://concepts/lock-value-relationships");

test_resource!(test_resource_cell_lifecycle, "ckb-dev-context://concepts-for-coding/cell-lifecycle");
test_resource!(test_resource_transaction_lifecycle, "ckb-dev-context://concepts-for-coding/transaction-lifecycle");

test_resource!(test_resource_binary_deployment, "ckb-dev-context://deployment/binary-deployment");
test_resource!(test_resource_cota_infrastructure, "ckb-dev-context://deployment/cota-infrastructure");

test_resource!(test_resource_project_directory, "ckb-dev-context://ecosystem/project-directory");

test_resource!(test_resource_interactive_courses, "ckb-dev-context://education/interactive-courses");

test_resource!(test_resource_developer_resources_and_tooling, "ckb-dev-context://getting-started/developer-resources-and-tooling");
test_resource!(test_resource_offckb_development_workflow, "ckb-dev-context://getting-started/offckb-development-workflow");
test_resource!(test_resource_tool_recommendations, "ckb-dev-context://getting-started/tool-recommendations");

test_resource!(test_resource_cell_collection_automation, "ckb-dev-context://integration-examples/cell-collection-automation");

test_resource!(test_resource_c_to_rust_script_migration, "ckb-dev-context://patterns/c-to-rust-script-migration");
test_resource!(test_resource_cota_nft_development, "ckb-dev-context://patterns/cota-nft-development");
test_resource!(test_resource_dao_development_patterns, "ckb-dev-context://patterns/dao-development-patterns");
test_resource!(test_resource_development_tools_and_templates, "ckb-dev-context://patterns/development-tools-and-templates");
test_resource!(test_resource_file_storage_patterns, "ckb-dev-context://patterns/file-storage-patterns");
test_resource!(test_resource_ickb_development, "ckb-dev-context://patterns/ickb-development");
test_resource!(test_resource_ickb_liquidity_patterns, "ckb-dev-context://patterns/ickb-liquidity-patterns");
test_resource!(test_resource_minimal_lock_script_full, "ckb-dev-context://patterns/minimal-lock-script");
test_resource!(test_resource_minimal_type_script, "ckb-dev-context://patterns/minimal-type-script");
test_resource!(test_resource_molecule_schema_development, "ckb-dev-context://patterns/molecule-schema-development");
test_resource!(test_resource_omnilock_development, "ckb-dev-context://patterns/omnilock-development");
test_resource!(test_resource_omnilock_interoperability, "ckb-dev-context://patterns/omnilock-interoperability");
test_resource!(test_resource_operation_detection, "ckb-dev-context://patterns/operation-detection");
test_resource!(test_resource_rust_script_development_patterns, "ckb-dev-context://patterns/rust-script-development-patterns");
test_resource!(test_resource_script_development_patterns, "ckb-dev-context://patterns/script-development-patterns");
test_resource!(test_resource_script_source_patterns, "ckb-dev-context://patterns/script-source-patterns");
test_resource!(test_resource_seed_cell_pattern, "ckb-dev-context://patterns/seed-cell-pattern");
test_resource!(test_resource_simple_transfer, "ckb-dev-context://patterns/simple-transfer");
test_resource!(test_resource_spore_development, "ckb-dev-context://patterns/spore-development");
test_resource!(test_resource_system_scripts_and_core_patterns, "ckb-dev-context://patterns/system-scripts-and-core-patterns");
test_resource!(test_resource_token_creation, "ckb-dev-context://patterns/token-creation");
test_resource!(test_resource_transaction_building_patterns, "ckb-dev-context://patterns/transaction-building-patterns");
test_resource!(test_resource_type_id_pattern, "ckb-dev-context://patterns/type-id-pattern");
test_resource!(test_resource_udt_tokens, "ckb-dev-context://patterns/udt-tokens");
test_resource!(test_resource_cobuild_integration, "ckb-dev-context://patterns/cobuild-integration");
test_resource!(test_resource_ssri_implementation, "ckb-dev-context://patterns/ssri-implementation");
test_resource!(test_resource_dob_development, "ckb-dev-context://patterns/dob-development");
test_resource!(test_resource_proxy_lock_patterns, "ckb-dev-context://patterns/proxy-lock-patterns");
test_resource!(test_resource_proxy_lock_testing_patterns, "ckb-dev-context://patterns/proxy-lock-testing-patterns");
test_resource!(test_resource_contract_workspace_development, "ckb-dev-context://patterns/contract-workspace-development");

test_resource!(test_resource_ckbfs_protocol, "ckb-dev-context://protocols/ckbfs-protocol");
test_resource!(test_resource_cota_protocol, "ckb-dev-context://protocols/cota-protocol");
test_resource!(test_resource_ickb_protocol_full, "ckb-dev-context://protocols/ickb-protocol");
test_resource!(test_resource_omnilock_protocol, "ckb-dev-context://protocols/omnilock-protocol");
test_resource!(test_resource_rgb_plus_plus, "ckb-dev-context://protocols/rgb-plus-plus");
test_resource!(test_resource_spore_digital_objects, "ckb-dev-context://protocols/spore-digital-objects");
test_resource!(test_resource_spore_protocol_full, "ckb-dev-context://protocols/spore-protocol");
test_resource!(test_resource_cobuild_protocol_full, "ckb-dev-context://protocols/cobuild");
test_resource!(test_resource_open_transaction, "ckb-dev-context://protocols/open-transaction");
test_resource!(test_resource_ssri_protocol, "ckb-dev-context://protocols/ssri");
test_resource!(test_resource_xudt_protocol_full, "ckb-dev-context://protocols/xudt-protocol");

test_resource!(test_resource_common_script_errors_full, "ckb-dev-context://troubleshooting/common-script-errors");
test_resource!(test_resource_ickb_debugging, "ckb-dev-context://troubleshooting/ickb-debugging");
test_resource!(test_resource_rust_script_development_issues, "ckb-dev-context://troubleshooting/rust-script-development-issues");
test_resource!(test_resource_omnilock_errors, "ckb-dev-context://troubleshooting/omnilock-errors");
test_resource!(test_resource_xudt_errors, "ckb-dev-context://troubleshooting/xudt-errors");
test_resource!(test_resource_transaction_building_errors, "ckb-dev-context://troubleshooting/transaction-building-errors");
test_resource!(test_resource_spore_errors, "ckb-dev-context://troubleshooting/spore-errors");

test_resource!(test_resource_ssri_server, "ckb-dev-context://tools/ssri-server");
