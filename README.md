# mcp-jsonrpc

A JSON-RPC 2.0 adapter for the MCP Core system, providing a standardized way to interact with MCP tools over various transports.

## Features

- Full [JSON-RPC 2.0](https://www.jsonrpc.org/specification) protocol support
- Async/await transport layer
- Built-in TCP and Unix domain socket support
- Batch request processing
- Notification support
- Comprehensive error handling and mapping
- Thread-safe and Clone-able for multi-connection servers

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
mcp-jsonrpc = "0.1.0"
```

### Basic Example

Here's a simple example of creating a TCP JSON-RPC server:

```rust
use mcp_jsonrpc::{JsonRpcProcessor, ToolRegistry, Tool, TcpTransport};
use serde_json::Value;
use async_trait::async_trait;

// Implement a simple echo tool
struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    async fn execute(&self, params: Value) -> Result<Value, String> {
        Ok(params)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and register tools
    let registry = ToolRegistry::builder()
        .with_tool("echo", EchoTool)
        .build();

    // Create TCP listener
    let listener = TcpTransport::bind("127.0.0.1:8080").await?;

    println!("Server listening on 127.0.0.1:8080");

    // Accept connections
    while let Ok((stream, _)) = listener.accept().await {
        let registry = registry.clone();
        
        tokio::spawn(async move {
            let mut processor = JsonRpcProcessor::new(stream, registry);
            if let Err(e) = processor.run().await {
                eprintln!("Client error: {}", e);
            }
        });
    }

    Ok(())
}
```

### Client Example

```rust
use mcp_jsonrpc::{TcpTransport, JsonRpcRequest, JsonRpcId};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the server
    let mut transport = TcpTransport::connect("127.0.0.1:8080").await?;

    // Create a request
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "echo".to_string(),
        params: Some(json!({"message": "Hello, world!"})),
        id: JsonRpcId::Number(1),
    };

    // Send request and receive response
    transport.send(&serde_json::to_string(&request)?).await?;
    let response = transport.receive().await?;
    println!("Response: {}", response);

    Ok(())
}
```

### Using Unix Domain Sockets

On Unix platforms, you can use Unix domain sockets instead of TCP:

```rust
use mcp_jsonrpc::UnixTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = UnixTransport::bind("/tmp/jsonrpc.sock").await?;
    // ... rest of the server code ...
}
```

### Notifications

Send notifications that don't require responses:

```rust
use mcp_jsonrpc::{JsonRpcNotification, TcpTransport};

let notification = JsonRpcNotification {
    jsonrpc: "2.0".to_string(),
    method: "log".to_string(),
    params: Some(json!({"level": "info", "message": "Hello!"})),
};

transport.send(&serde_json::to_string(&notification)?).await?;
```

### Batch Requests

Process multiple requests in a single call:

```rust
let requests = vec![
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "echo".to_string(),
        params: Some(json!({"first": true})),
        id: JsonRpcId::Number(1),
    },
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "echo".to_string(),
        params: Some(json!({"second": true})),
        id: JsonRpcId::Number(2),
    },
];

transport.send(&serde_json::to_string(&requests)?).await?;
```

## Error Handling

The crate maps domain errors to appropriate JSON-RPC error codes:

| Domain Error   | JSON-RPC Error Code | Description     |
|---------------|---------------------|-----------------|
| ToolNotFound  | -32601             | Method not found|
| InvalidParams | -32602             | Invalid params  |
| ToolError     | -32000             | Server error    |
| InternalError | -32603             | Internal error  |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
