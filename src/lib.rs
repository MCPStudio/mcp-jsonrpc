//! MCP JSON-RPC Adapter
//!
//! This crate provides a JSON-RPC 2.0 adapter for the MCP Core system. Its primary
//! responsibility is to translate between domain messages defined in `mcp-core` and
//! JSON-RPC protocol messages, allowing clients to interact with the MCP system using
//! the standardized JSON-RPC protocol.
//!
//! # Responsibilities
//!
//! - Define JSON-RPC 2.0 protocol structures
//! - Validate JSON-RPC messages according to the specification
//! - Convert between domain messages and JSON-RPC messages
//! - Map domain errors to appropriate JSON-RPC error codes
//!
//! # Non-responsibilities
//!
//! - Transport layer implementation (belongs in transport-specific crates)
//! - Business logic (belongs in `mcp-core`)
//! - Server orchestration (belongs in `mcp-server`)
//!
//! # Example
//!
//! ```rust,no_run
//! use mcp_jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcId};
//! use serde_json::{json, Value};
//!
//! // Parse a JSON-RPC request
//! let request_str = r#"{"jsonrpc":"2.0","method":"echo","params":{"text":"Hello"},"id":1}"#;
//! let request: JsonRpcRequest = serde_json::from_str(request_str).unwrap();
//!
//! // Validate the request
//! assert!(request.validate().is_ok());
//!
//! // Convert to domain request (in a real implementation, this would use mcp-core types)
//! let domain_request = mcp_jsonrpc::conversion::json_rpc_to_domain_request(&request).unwrap();
//!
//! // Create a domain response (simplified example)
//! let domain_response = mcp_jsonrpc::conversion::SimpleDomainResponse {
//!     id: domain_request.id.to_string(),
//!     result: Ok(json!({"text": "Hello"})),
//! };
//!
//! // Convert back to JSON-RPC response
//! let response = mcp_jsonrpc::conversion::domain_to_json_rpc_response(&domain_response).unwrap();
//!
//! // Serialize to send to client
//! let response_str = serde_json::to_string(&response).unwrap();
//! ```

// Publicly expose the core JSON-RPC protocol structures
pub mod conversion;
pub mod error;
pub mod protocol;

// Re-export core types for convenience
pub use protocol::{
    JsonRpcBatch, JsonRpcError, JsonRpcId, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};

// Re-export error types
pub use mcp_error::{EphErrorExt, Error as McpError, OrExit, Result, Severity, Result as McpResult};

// Keep these modules but mark them as in transition
// TODO: These will be moved to their own crates in the future
#[doc(hidden)]
pub mod processor;
#[doc(hidden)]
pub mod transport;

// Re-export for backward compatibility (to be removed in future)
#[doc(hidden)]
pub use processor::{JsonRpcProcessor, Tool, ToolRegistry};
#[doc(hidden)]
pub use transport::base::{JsonRpcTransport, Transport};
#[doc(hidden)]
pub use transport::{TcpTransport, UnixTransport};
