# MCP-JSONRPC Implementation Guide

This document provides guidelines for implementing the `mcp-jsonrpc` crate, which will serve as an adapter for the `mcp-core` domain following hexagonal architecture principles.

## Project Purpose

The `mcp-jsonrpc` crate will implement the [JSON-RPC 2.0](https://www.jsonrpc.org/specification) protocol as an adapter for the MCP Core system. This adapter will translate between domain messages defined in `mcp-core` and JSON-RPC protocol messages, allowing clients to interact with the MCP system using the standardized JSON-RPC protocol.

## Architectural Context

This project follows hexagonal architecture (also known as ports and adapters):

1. `mcp-core` is the **domain layer** containing business logic and ports (interfaces)
2. `mcp-jsonrpc` is an **adapter** that implements specific technology details (JSON-RPC)

Key principles to follow:
- Dependencies flow from outside (adapters) to inside (domain)
- The domain knows nothing about adapters
- Communication occurs through ports (interfaces) defined by the domain

## Implementation Requirements

### 1. JSON-RPC 2.0 Protocol Structures

Implement the standard JSON-RPC 2.0 message types:

```rust
/// JSON-RPC 2.0 Request object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,
    
    /// A String containing the name of the method to be invoked.
    pub method: String,
    
    /// A Structured value that holds the parameter values to be used during the invocation of the method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    
    /// An identifier established by the Client that MUST contain a String, Number, or NULL value if included.
    pub id: JsonRpcId,
}

/// JSON-RPC 2.0 Response object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,
    
    /// This member is REQUIRED on success. This member MUST NOT exist if there was an error invoking the method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    /// This member is REQUIRED on error. This member MUST NOT exist if there was no error triggered during invocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    
    /// This member is REQUIRED. It MUST be the same as the value of the id member in the Request Object.
    pub id: JsonRpcId,
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
    pub params: Option<serde_json::Value>,
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
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC ID (can be string, number, or null)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Null,
    String(String),
    Number(i64),
}
```

### 2. Adapters for Domain Conversion

Implement converters between domain messages and JSON-RPC messages:

```rust
// Example converter functions (implement actual logic)
impl JsonRpcRequest {
    pub fn to_domain_request(&self) -> Result<mcp_core::Request, ConversionError> {
        // Convert JsonRpcRequest to domain Request
        Ok(mcp_core::Request {
            id: self.id.to_string(),
            tool_name: self.method.clone(),
            params: self.params.clone().unwrap_or(serde_json::Value::Null),
        })
    }
}

impl mcp_core::Response {
    pub fn to_json_rpc(&self) -> JsonRpcResponse {
        // Convert domain Response to JsonRpcResponse
        match &self.result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value.clone()),
                error: None,
                id: parse_id(&self.id),
            },
            Err(err) => {
                let (code, message) = error_to_json_rpc(err);
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code,
                        message,
                        data: Some(json!({ "error_type": format!("{:?}", err) })),
                    }),
                    id: parse_id(&self.id),
                }
            }
        }
    }
}
```

### 3. Transport Implementation

Implement the `Transport` port from `mcp-core` to handle JSON-RPC communication:

```rust
pub struct JsonRpcTransport<T> {
    inner: T,
    // Add any state needed
}

#[async_trait]
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Transport for JsonRpcTransport<T> {
    async fn receive(&mut self) -> Result<String, TransportError> {
        // Read raw data from transport
        // Parse as JSON-RPC
        // Return the raw JSON string
    }

    async fn send(&mut self, message: &str) -> Result<(), TransportError> {
        // Send the message over the transport
    }
}
```

### 4. Message Processor

Implement a processor that uses `MCP` from the core domain:

```rust
pub struct JsonRpcProcessor<T: Transport> {
    mcp: MCP<JsonRpcTransport<T>>,
}

impl<T: Transport> JsonRpcProcessor<T> {
    pub fn new(transport: T, tools: ToolRegistry) -> Self {
        let json_rpc_transport = JsonRpcTransport::new(transport);
        let mcp = MCPBuilder::new()
            // Register tools
            .build(json_rpc_transport);
        
        Self { mcp }
    }
    
    pub async fn run(&mut self) -> Result<(), TransportError> {
        // Custom implementation that:
        // 1. Receives JSON-RPC message via transport
        // 2. Converts it to domain Request
        // 3. Processes the request using mcp.process_request()
        // 4. Converts the domain Response back to JSON-RPC
        // 5. Sends the response via transport
    }
}
```

## Error Handling

Implement proper mapping between domain errors and JSON-RPC error codes:

| Domain Error | JSON-RPC Error Code | Description |
|--------------|---------------------|-------------|
| ToolNotFound | -32601 | Method not found |
| InvalidParams | -32602 | Invalid params |
| ToolError | -32000 | Server error |
| InternalError | -32603 | Internal error |

## Testing Requirements

1. Unit tests for all message conversions
2. Tests for proper JSON-RPC protocol validation
3. Integration tests with mock tools
4. Error handling tests

## Common Pitfalls to Avoid

1. **Don't modify the domain**: The adapter should conform to the domain, not vice versa
2. **Follow JSON-RPC spec strictly**: Ensure compliance with the specification
3. **Proper error handling**: Map domain errors to appropriate JSON-RPC error codes
4. **Handle edge cases**: Batch requests, notifications, malformed messages
5. **Avoid leaking implementation details**: Keep JSON-RPC details isolated in this adapter

## Dependencies

- `mcp-core`: The domain crate
- `serde` & `serde_json`: For JSON serialization
- `async-trait`: For async trait support
- `tokio`: For async runtime (if needed)
- Any transport-specific dependencies (WebSockets, HTTP, etc.)

## Example Usage

```rust
// Example implementation with WebSockets
async fn main() {
    // Set up tools
    let mut tool_registry = ToolRegistry::new();
    tool_registry.insert("echo", Box::new(EchoTool::new()));
    
    // Set up WebSocket transport (implementation detail)
    let ws_stream = connect_websocket("ws://localhost:8080").await.unwrap();
    
    // Create processor with transport and tools
    let mut processor = JsonRpcProcessor::new(ws_stream, tool_registry);
    
    // Run the processor
    processor.run().await.unwrap();
}
```