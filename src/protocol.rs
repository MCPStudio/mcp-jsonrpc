use crate::error::helpers;
use mcp_error::Result as McpResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,

    /// A String containing the name of the method to be invoked.
    pub method: String,

    /// A Structured value that holds the parameter values to be used during the invocation of the method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    /// An identifier established by the Client that MUST contain a String, Number, or NULL value if included.
    pub id: JsonRpcId,
}

impl JsonRpcRequest {
    /// Validate that a request adheres to the JSON-RPC 2.0 specification
    pub fn validate(&self) -> McpResult<()> {
        // Check protocol version
        if self.jsonrpc != "2.0" {
            return Err(helpers::protocol_error(
                "Invalid JSON-RPC version. Must be exactly '2.0'",
            ));
        }

        // Method must not be empty
        if self.method.is_empty() {
            return Err(helpers::protocol_error("Method must not be empty"));
        }

        // Method must not start with 'rpc.'
        if self.method.starts_with("rpc.") {
            return Err(helpers::protocol_error(
                "Method names that begin with 'rpc.' are reserved",
            ));
        }

        Ok(())
    }
}

/// JSON-RPC 2.0 Response object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,

    /// This member is REQUIRED on success. This member MUST NOT exist if there was an error invoking the method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// This member is REQUIRED on error. This member MUST NOT exist if there was no error triggered during invocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object.
    pub id: JsonRpcId,
}

impl JsonRpcResponse {
    /// Validate that a response adheres to the JSON-RPC 2.0 specification
    pub fn validate(&self) -> McpResult<()> {
        // Check protocol version
        if self.jsonrpc != "2.0" {
            return Err(helpers::protocol_error(
                "Invalid JSON-RPC version. Must be exactly '2.0'",
            ));
        }

        // Result and error are mutually exclusive
        match (&self.result, &self.error) {
            (Some(_), Some(_)) => {
                return Err(helpers::protocol_error(
                    "Response cannot contain both result and error",
                ));
            }
            (None, None) => {
                return Err(helpers::protocol_error(
                    "Response must contain either result or error",
                ));
            }
            _ => {}
        }

        // Validate error object if present
        if let Some(error) = &self.error {
            error.validate()?;
        }

        Ok(())
    }
}

/// JSON-RPC 2.0 Notification (a request without an ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,

    /// A String containing the name of the method to be invoked.
    pub method: String,

    /// A Structured value that holds the parameter values to be used during the invocation of the method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    /// Validate that a notification adheres to the JSON-RPC 2.0 specification
    pub fn validate(&self) -> McpResult<()> {
        // Check protocol version
        if self.jsonrpc != "2.0" {
            return Err(helpers::protocol_error(
                "Invalid JSON-RPC version. Must be exactly '2.0'",
            ));
        }

        // Method must not be empty
        if self.method.is_empty() {
            return Err(helpers::protocol_error("Method must not be empty"));
        }

        // Method must not start with 'rpc.'
        if self.method.starts_with("rpc.") {
            return Err(helpers::protocol_error(
                "Method names that begin with 'rpc.' are reserved",
            ));
        }

        Ok(())
    }
}

/// JSON-RPC 2.0 Error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// A Number that indicates the error type that occurred.
    pub code: i32,

    /// A String providing a short description of the error.
    pub message: String,

    /// A Primitive or Structured value that contains additional information about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Validate that an error object adheres to the JSON-RPC 2.0 specification
    pub fn validate(&self) -> McpResult<()> {
        // Error message must not be empty
        if self.message.is_empty() {
            return Err(helpers::protocol_error("Error message must not be empty"));
        }

        // Validate error code ranges
        match self.code {
            -32700 => {}                                    // Parse error
            -32600 => {}                                    // Invalid request
            -32601 => {}                                    // Method not found
            -32602 => {}                                    // Invalid params
            -32603 => {}                                    // Internal error
            code if (-32099..=-32000).contains(&code) => {} // Server error
            _ => {
                return Err(helpers::protocol_error(&format!(
                    "Invalid error code: {}",
                    self.code
                )));
            }
        }

        Ok(())
    }
}

/// JSON-RPC ID (can be string, number, or null)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[derive(PartialEq)]
pub enum JsonRpcId {
    Null,
    String(String),
    Number(i64),
}

impl JsonRpcId {
    /// Convert the JsonRpcId to a string representation
    pub fn to_string(&self) -> String {
        match self {
            JsonRpcId::Null => "null".to_string(),
            JsonRpcId::String(s) => s.clone(),
            JsonRpcId::Number(n) => n.to_string(),
        }
    }
}

/// Batch of JSON-RPC requests/notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcBatch {
    /// Multiple requests
    Requests(Vec<JsonRpcRequest>),
    /// Multiple notifications
    Notifications(Vec<JsonRpcNotification>),
}

/// Parse a string ID into a JsonRpcId
pub fn parse_id(id_str: &str) -> JsonRpcId {
    if id_str == "null" {
        return JsonRpcId::Null;
    }

    // Try to parse as number first
    if let Ok(num) = id_str.parse::<i64>() {
        return JsonRpcId::Number(num);
    }

    // Otherwise, treat as string
    JsonRpcId::String(id_str.to_string())
}
