//! HTTP server setup with rmcp Streamable HTTP transport.

use anyhow::Result;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::{LocalSessionManager, SessionConfig};
use serde::Serialize;
use shared::ckb_client::CkbRpcClient;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::ServerConfig;
use crate::capabilities::CkbMcpServerFactory;
use crate::dev::DevHandlers;
use crate::jsonrpc::jsonrpc_handler;
use crate::middleware::DeferLoadingLayer;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
	pub config: ServerConfig,
	pub dev_handlers: Option<Arc<DevHandlers>>,
	pub docs_handlers: Option<Arc<crate::docs::DocsHandlers>>,
}

/// Response from the file upload endpoint.
#[derive(Serialize)]
struct UploadResponse {
	tx_hash: String,
	output_index: u32,
	data_size: usize,
	capacity: u64,
}

/// Error response from the file upload endpoint.
#[derive(Serialize)]
struct UploadError {
	error: String,
}

/// Run the MCP server.
pub async fn run(addr: SocketAddr, config: ServerConfig) -> Result<()> {
	// Create dev handlers if tools are enabled.
	let dev_handlers = if config.args.tools_enabled() {
		match CkbRpcClient::new(&config.args.ckb_rpc) {
			Ok(client) => match DevHandlers::new(
				client,
				config.args.ckb_rpc.clone(),
				config.args.private_key.clone(),
			) {
				Ok(handlers) => Some(Arc::new(handlers)),
				Err(e) => {
					error!("Failed to create dev handlers for file upload: {}", e);
					None
				}
			},
			Err(e) => {
				error!("Failed to create CKB client for file upload: {}", e);
				None
			}
		}
	} else {
		None
	};

	// Create the MCP service factory with shared dev handlers.
	let factory = CkbMcpServerFactory::new(config.clone(), dev_handlers.clone());

	let state = AppState {
		config: config.clone(),
		dev_handlers,
		docs_handlers: factory.docs_handlers(),
	};

	let session_manager = create_session_manager();

	let mcp_service = StreamableHttpService::new(
		move || factory.create(),
		session_manager.into(),
		Default::default(),
	);

	// Wrap MCP service with DeferLoadingLayer to inject defer_loading property.
	let mcp_service_with_defer = ServiceBuilder::new()
		.layer(DeferLoadingLayer)
		.service(mcp_service);

	// Build the router.
	let app = Router::new()
		// Health endpoint.
		.route("/health", get(health_handler))
		// Stats endpoint.
		.route("/stats", get(stats_handler))
		// File upload endpoint for large deployments.
		.route("/deploy/file", post(upload_file_handler))
		// JSON-RPC endpoint for plain HTTP requests.
		.route("/rpc", post(jsonrpc_handler))
		// MCP endpoint via StreamableHttpService with defer_loading injection.
		.nest_service("/mcp", mcp_service_with_defer)
		// Shared state.
		.with_state(Arc::new(state))
		// CORS configuration.
		.layer(
			CorsLayer::new()
				.allow_origin(Any)
				.allow_methods(Any)
				.allow_headers(Any),
		)
		// Request tracing.
		.layer(TraceLayer::new_for_http());

	info!("Server starting on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await?;
	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await?;

	info!("Server shutdown complete");
	Ok(())
}

fn create_session_manager() -> LocalSessionManager {
	LocalSessionManager {
		session_config: SessionConfig {
			channel_capacity: SessionConfig::DEFAULT_CHANNEL_CAPACITY,
			// Do not expire normal MCP client sessions while the app is idle.
			// Codex and Claude Code reuse Mcp-Session-Id across user turns.
			keep_alive: None,
		},
		..Default::default()
	}
}

/// Health check endpoint.
async fn health_handler() -> impl IntoResponse {
	(StatusCode::OK, "OK")
}

/// Server version from Cargo.toml.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stats endpoint with format query parameter.
async fn stats_handler(
	State(state): State<Arc<AppState>>,
	axum::extract::Query(params): axum::extract::Query<StatsQueryParams>,
) -> impl IntoResponse {
	let format = params.format.as_deref().unwrap_or("human");

	let result = match format {
		"json" => state.config.stats.format_json(Some(VERSION)),
		"prometheus" => state.config.stats.format_prometheus(Some(VERSION)),
		_ => state.config.stats.format_human(Some(VERSION)),
	};

	match result {
		Ok(body) => {
			let content_type = match format {
				"json" => "application/json",
				"prometheus" => "text/plain; charset=utf-8",
				_ => "text/plain; charset=utf-8",
			};
			(
				StatusCode::OK,
				[(axum::http::header::CONTENT_TYPE, content_type)],
				body,
			)
				.into_response()
		}
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
	}
}

#[derive(Debug, serde::Deserialize)]
struct StatsQueryParams {
	format: Option<String>,
}

/// Shutdown signal handler.
async fn shutdown_signal() {
	let ctrl_c = async {
		tokio::signal::ctrl_c()
			.await
			.expect("Failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
			.expect("Failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {
			info!("Received Ctrl+C, shutting down");
		}
		_ = terminate => {
			info!("Received SIGTERM, shutting down");
		}
	}
}

/// File upload handler for deploying large files.
async fn upload_file_handler(
	State(state): State<Arc<AppState>>,
	mut multipart: Multipart,
) -> std::result::Result<Json<UploadResponse>, (StatusCode, Json<UploadError>)> {
	// Check if dev tools are enabled.
	let dev_handlers = state.dev_handlers.as_ref().ok_or_else(|| {
		(
			StatusCode::SERVICE_UNAVAILABLE,
			Json(UploadError {
				error:
					"Dev tools are not enabled. Use --tools-only or ensure tools are not disabled."
						.to_string(),
			}),
		)
	})?;

	// Extract file from multipart form.
	let mut file_data: Option<Vec<u8>> = None;

	while let Some(field) = multipart.next_field().await.map_err(|e| {
		(
			StatusCode::BAD_REQUEST,
			Json(UploadError {
				error: format!("Failed to read multipart field: {}", e),
			}),
		)
	})? {
		let name = field.name().unwrap_or("").to_string();
		if name == "file" {
			let data = field.bytes().await.map_err(|e| {
				(
					StatusCode::BAD_REQUEST,
					Json(UploadError {
						error: format!("Failed to read file data: {}", e),
					}),
				)
			})?;
			file_data = Some(data.to_vec());
			break;
		}
	}

	let data = file_data.ok_or_else(|| {
		(
			StatusCode::BAD_REQUEST,
			Json(UploadError {
				error: "No 'file' field found in multipart form. Use: curl -F 'file=@/path/to/file' <url>/deploy/file".to_string(),
			}),
		)
	})?;

	info!("Received file upload: {} bytes", data.len());

	// Deploy using dev handlers.
	let result = dev_handlers.deploy_cell_data(data).await.map_err(|e| {
		error!("File deployment failed: {}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(UploadError {
				error: format!("Deployment failed: {}", e),
			}),
		)
	})?;

	info!(
		"File deployed successfully: tx_hash={}, output_index={}",
		result.tx_hash, result.output_index
	);

	Ok(Json(UploadResponse {
		tx_hash: result.tx_hash,
		output_index: result.output_index,
		data_size: result.data_size,
		capacity: result.capacity,
	}))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::capabilities::CkbMcpServer;
	use crate::docs::DocsHandlers;
	use axum::body::Body;
	use axum::http::{Method, Request};
	use rmcp::model::{
		CallToolRequestMethod, CallToolRequestParams, ClientCapabilities, ClientJsonRpcMessage,
		ClientNotification, ClientRequest, Implementation, InitializeRequestParams,
		InitializedNotification, JsonObject, NumberOrString, ProtocolVersion,
	};
	use rmcp::transport::streamable_http_server::StreamableHttpService;
	use rmcp::transport::streamable_http_server::session::local::SessionConfig;
	use serde_json::json;
	use std::path::PathBuf;
	use std::time::Duration;
	use tower::Service;

	#[test]
	fn mcp_sessions_do_not_expire_while_client_is_idle() {
		let session_manager = create_session_manager();

		assert_eq!(session_manager.session_config.keep_alive, None);
	}

	#[tokio::test]
	async fn expired_mcp_session_rejects_reused_session_id() {
		let session_manager = LocalSessionManager {
			session_config: SessionConfig {
				channel_capacity: SessionConfig::DEFAULT_CHANNEL_CAPACITY,
				keep_alive: Some(Duration::from_millis(20)),
			},
			..Default::default()
		};
		let mut service: StreamableHttpService<CkbMcpServer, LocalSessionManager> =
			StreamableHttpService::new(
				|| Ok(test_mcp_server()),
				session_manager.into(),
				Default::default(),
			);

		let init_response = service
			.call(mcp_post_request(initialize_message(), None))
			.await
			.expect("initialize request should return a response");
		assert_eq!(init_response.status(), StatusCode::OK);
		let session_id = init_response
			.headers()
			.get("mcp-session-id")
			.expect("initialize response should include session id")
			.to_str()
			.expect("session id should be valid")
			.to_string();

		let initialized_response = service
			.call(mcp_post_request(initialized_message(), Some(&session_id)))
			.await
			.expect("initialized notification should return a response");
		assert_eq!(initialized_response.status(), StatusCode::ACCEPTED);

		tokio::time::sleep(Duration::from_millis(80)).await;

		let stale_response = service
			.call(mcp_post_request(
				search_resources_message(),
				Some(&session_id),
			))
			.await
			.expect("stale session request should return a response");

		assert_eq!(stale_response.status(), StatusCode::UNAUTHORIZED);
	}

	fn test_mcp_server() -> CkbMcpServer {
		let dir = tempfile::tempdir().expect("tempdir should be created");
		let stats = Arc::new(
			shared::stats::Stats::open(dir.path().join("stats.redb")).expect("stats should open"),
		);
		let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs");
		let docs_handlers =
			Arc::new(DocsHandlers::new(docs_path.clone()).expect("docs handlers should load docs"));
		let config = ServerConfig {
			args: crate::Args {
				port: 0,
				host: "127.0.0.1".to_string(),
				ckb_rpc: "http://127.0.0.1:8114".to_string(),
				private_key: "0x6109170b275a09ad54877b82f7d9930f88cab5717d484fb4741c9f0c0571c078"
					.to_string(),
				docs_path,
				stats_db: "unused.redb".into(),
				log_level: "error".to_string(),
				docs_only: true,
				rpc_only: false,
				tools_only: false,
				no_prompts: false,
			},
			stats,
		};

		CkbMcpServer::new_with_handlers(config, None, Some(docs_handlers))
	}

	fn mcp_post_request(message: ClientJsonRpcMessage, session_id: Option<&str>) -> Request<Body> {
		let body = serde_json::to_vec(&message).expect("message should serialize");
		let mut builder = Request::builder()
			.method(Method::POST)
			.uri("/mcp")
			.header("accept", "application/json, text/event-stream")
			.header("content-type", "application/json");

		if let Some(session_id) = session_id {
			builder = builder.header("mcp-session-id", session_id);
		}

		builder
			.body(Body::from(body))
			.expect("request should be built")
	}

	fn initialize_message() -> ClientJsonRpcMessage {
		ClientJsonRpcMessage::request(
			ClientRequest::InitializeRequest(rmcp::model::Request::new(InitializeRequestParams {
				meta: None,
				protocol_version: ProtocolVersion::V_2025_06_18,
				capabilities: ClientCapabilities::default(),
				client_info: Implementation {
					name: "idle-session-test".to_string(),
					title: None,
					version: "0.0.0".to_string(),
					icons: None,
					website_url: None,
				},
			})),
			NumberOrString::Number(1),
		)
	}

	fn initialized_message() -> ClientJsonRpcMessage {
		ClientJsonRpcMessage::notification(ClientNotification::InitializedNotification(
			InitializedNotification::default(),
		))
	}

	fn search_resources_message() -> ClientJsonRpcMessage {
		let mut arguments = JsonObject::new();
		arguments.insert("query".to_string(), json!("cell"));

		ClientJsonRpcMessage::request(
			ClientRequest::CallToolRequest(rmcp::model::Request {
				method: CallToolRequestMethod,
				params: CallToolRequestParams {
					meta: None,
					name: "search_resources".into(),
					arguments: Some(arguments),
					task: None,
				},
				extensions: Default::default(),
			}),
			NumberOrString::Number(2),
		)
	}
}
