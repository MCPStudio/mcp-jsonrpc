use crate::protocol::{parse_id, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use mcp_error::{Error as McpError, Result as McpResult, Severity};
use serde_json::{json, Value};

/// A trait representing a domain request from mcp-core
/// This trait defines the interface for domain requests that will be used
/// when converting between JSON-RPC and domain messages
pub trait DomainRequest {
    /// Get the request identifier
    fn id(&self) -> &str;

    /// Get the name of the tool to be invoked
    fn tool_name(&self) -> &str;

    /// Get the parameters for the tool
    fn params(&self) -> &Value;
}

/// A trait representing a domain response from mcp-core
/// This trait defines the interface for domain responses that will be used
/// when converting between JSON-RPC and domain messages
pub trait DomainResponse {
    /// Get the response identifier (should match the request id)
    fn id(&self) -> &str;

    /// Get the result of the operation (success or error)
    fn result(&self) -> &McpResult<Value>;
}

/// A simple implementation of the DomainRequest trait for testing
/// In a real implementation, this would be replaced by the actual mcp-core types
#[doc(hidden)]
#[derive(Debug)]
pub struct SimpleDomainRequest {
    /// The request identifier
    pub id: String,

    /// The name of the tool to be invoked
    pub tool_name: String,

    /// The parameters for the tool
    pub params: Value,
}

impl DomainRequest for SimpleDomainRequest {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_name(&self) -> &str {
        &self.tool_name
    }

    fn params(&self) -> &Value {
        &self.params
    }
}

/// A simple implementation of the DomainResponse trait for testing
/// In a real implementation, this would be replaced by the actual mcp-core types
#[doc(hidden)]
pub struct SimpleDomainResponse {
    /// The response identifier
    pub id: String,

    /// The result of the operation
    pub result: McpResult<Value>,
}

impl DomainResponse for SimpleDomainResponse {
    fn id(&self) -> &str {
        &self.id
    }

    fn result(&self) -> &McpResult<Value> {
        &self.result
    }
}

/// Convert a JSON-RPC request to a domain request
///
/// This function validates the JSON-RPC request and creates a domain request object
/// that can be used by the core MCP system.
pub fn json_rpc_to_domain_request(req: &JsonRpcRequest) -> McpResult<SimpleDomainRequest> {
    // Ensure the request is valid according to the JSON-RPC spec
    req.validate()?;

    // Create the domain request
    Ok(SimpleDomainRequest {
        id: req.id.to_string(),
        tool_name: req.method.clone(),
        params: req.params.clone().unwrap_or(Value::Null),
    })
}

/// Convert a domain response to a JSON-RPC response
///
/// This function takes a domain response and creates a valid JSON-RPC response
/// that can be sent back to the client.
pub fn domain_to_json_rpc_response<T: DomainResponse>(resp: &T) -> McpResult<JsonRpcResponse> {
    match resp.result() {
        Ok(value) => {
            // Success response
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value.clone()),
                error: None,
                id: parse_id(resp.id()),
            };

            // Ensure the response is valid
            response.validate()?;

            Ok(response)
        }
        Err(err) => {
            // Create an McpError from the string error
            let domain_error = McpError::new(Severity::Error, "TOOL-ERROR", err.to_string());

            // Map the domain error to a JSON-RPC error code
            let (code, message) = crate::error::error_to_json_rpc(&domain_error);

            // Create the error response
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code,
                    message,
                    data: Some(json!({ "error_type": err })),
                }),
                id: parse_id(resp.id()),
            };

            // Ensure the response is valid
            response.validate()?;

            Ok(response)
        }
    }
}
