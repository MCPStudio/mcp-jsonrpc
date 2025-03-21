use mcp_error::{Error as McpError, Result as McpResult, Severity};

/// A specialized Result type for JSON-RPC operations
pub type Result<T> = McpResult<T>;

/// JSON-RPC error codes as defined in the specification
pub mod error_codes {
    /// Invalid JSON was received by the server.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Reserved for implementation-defined server-errors.
    pub const SERVER_ERROR_START: i32 = -32000;
    pub const SERVER_ERROR_END: i32 = -32099;
}

/// Reference codes for JSON-RPC adapter errors
pub mod reference_codes {
    /// JSON serialization/deserialization errors
    pub const JSON: &str = "JSONRPC-001";
    /// Transport-related errors
    pub const TRANSPORT: &str = "JSONRPC-002";
    /// Conversion between domain and JSON-RPC formats
    pub const CONVERSION: &str = "JSONRPC-003";
    /// JSON-RPC protocol violations
    pub const PROTOCOL: &str = "JSONRPC-004";
    /// Internal errors
    pub const INTERNAL: &str = "JSONRPC-005";
    /// Domain errors from mcp-core
    pub const DOMAIN: &str = "JSONRPC-006";
}

/// Domain error reference codes
/// These are expected to come from mcp-core in a real implementation
/// but are defined here for completeness
pub mod domain_reference_codes {
    /// Tool not found error
    pub const TOOL_NOT_FOUND: &str = "TOOL-NOTFOUND";
    /// Invalid parameters error
    pub const INVALID_PARAMS: &str = "PARAM-INVALID";
    /// Tool execution error
    pub const TOOL_ERROR: &str = "TOOL-ERROR";
    /// Internal error
    pub const INTERNAL: &str = "INTERNAL";
}

/// Convert from domain error to JSON-RPC error codes
///
/// This function maps domain error reference codes to the appropriate JSON-RPC error codes
/// according to the specification.
pub fn error_to_json_rpc(err: &McpError) -> (i32, String) {
    // Extract reference code from the error to determine JSON-RPC error code
    let reference = &err.reference;

    // Map domain error reference codes to JSON-RPC error codes
    match reference {
        ref_code if ref_code.contains(domain_reference_codes::TOOL_NOT_FOUND) => (
            error_codes::METHOD_NOT_FOUND,
            "Method not found".to_string(),
        ),

        ref_code if ref_code.contains(domain_reference_codes::INVALID_PARAMS) => {
            (error_codes::INVALID_PARAMS, "Invalid params".to_string())
        }

        ref_code if ref_code.contains(domain_reference_codes::TOOL_ERROR) => {
            (error_codes::SERVER_ERROR_START, "Server error".to_string())
        }

        ref_code
            if ref_code.contains(domain_reference_codes::INTERNAL)
                || ref_code.contains(reference_codes::INTERNAL) =>
        {
            (error_codes::INTERNAL_ERROR, "Internal error".to_string())
        }

        ref_code if ref_code.contains(reference_codes::JSON) => {
            (error_codes::PARSE_ERROR, "Parse error".to_string())
        }

        ref_code if ref_code.contains(reference_codes::PROTOCOL) => {
            (error_codes::INVALID_REQUEST, "Invalid Request".to_string())
        }

        _ => (
            error_codes::INTERNAL_ERROR,
            format!("Internal error: {}", err),
        ),
    }
}

/// Helper functions for creating common error types
pub mod helpers {
    use super::*;
    use std::error::Error as StdError;

    /// Convert a JSON error to a McpError
    pub fn json_error<E: StdError + 'static + std::marker::Sync + std::marker::Send>(err: E) -> McpError {
        McpError::new(
            Severity::Error,
            reference_codes::JSON,
            "JSON processing error",
        )
        .with_source(Box::new(err))
    }

    /// Create a transport error
    pub fn transport_error(msg: &str) -> McpError {
        McpError::new(Severity::Error, reference_codes::TRANSPORT, msg)
    }

    /// Create a conversion error
    pub fn conversion_error(msg: &str) -> McpError {
        McpError::new(Severity::Error, reference_codes::CONVERSION, msg)
    }

    /// Create a protocol error
    pub fn protocol_error(msg: &str) -> McpError {
        McpError::new(Severity::Error, reference_codes::PROTOCOL, msg)
    }

    /// Create an internal error
    pub fn internal_error(msg: &str) -> McpError {
        McpError::new(Severity::Critical, reference_codes::INTERNAL, msg)
    }

    /// Wrap a domain error
    pub fn domain_error<E: StdError + 'static + std::marker::Sync + std::marker::Send>(err: E, msg: &str) -> McpError {
        McpError::new(Severity::Error, reference_codes::DOMAIN, msg).with_source(Box::new(err))
    }
}
