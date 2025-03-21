# MCP Core System Architecture

## Overview
The MCP Core system is designed with modularity in mind, separating concerns into distinct crates with specific responsibilities. This architecture promotes flexibility, reusability, and maintainability.

## Core Components

### `mcp-core` (Library)
- Defines **abstract domain data structures** (requests, responses, error enums, etc.) independent of any specific wire protocol
- Declares essential system traits ("ports"):
  - `Transport` for receiving/sending lines (independent of STDIO, TCP, SSE, etc.)
  - `Tool` for implementing functionality (HTTP calls, shell commands, local functions, etc.)
- May include a simple `ToolRegistry` for registering/listing/finding tools by name
- **Does not implement JSON-RPC** details (like "jsonrpc": "2.0"), leaving those specifics to `mcp-jsonrpc`
- Contains no concrete I/O or business logic
- Minimal dependencies (only standard crates like tokio, serde, possibly async-trait)

### `mcp-jsonrpc` (Library)
- An adapter that knows about **JSON-RPC 2.0** and how to translate raw JSON to/from domain types from `mcp-core`
- Parses & validates JSON-RPC messages:
  - Converts raw JSON (e.g. {"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}) into abstract requests/notifications understood by `mcp-core`
  - Handles JSON-RPC fields like "jsonrpc", "id", "method", "params" to differentiate requests vs. notifications
  - Maps JSON-RPC error codes (e.g. -32601) to/from domain errors (`McpError` in `mcp-core`)
- Serializes domain responses back into JSON:
  - Takes an abstract response (e.g. `Response::Success`) and produces {"jsonrpc":"2.0","id":...,"result":...}
  - Handles error responses (e.g. {"jsonrpc":"2.0","id":...,"error":{...}})
- Optionally provides helper logic for routing "method" names (like "tools/list") to correct `mcp-core` request variants
- Leaves all I/O to transport crates; deals only with string <-> domain conversions

### `mcp-transport-stdio` (Library)
- Implements a `Transport` adapter (from `mcp-core`) specifically for STDIN/STDOUT
- Handles reading JSON messages line by line from stdin and writing JSON responses to stdout
- Manages specifics around buffering, partial reads, newlines, and async operations (via Tokio)
- Depends on `mcp-core` but not on server logic

### `mcp-tool-http` (Library)
- Demonstrates implementation of a `Tool` for making HTTP calls (using reqwest)
- Exposes structs like `HttpGetTool` that implement the `Tool` trait
- Processes parameters (e.g., {"url":"..."}) and returns JSON data from remote APIs
- Depends on `mcp-core` for the `Tool` trait and reqwest for network calls

### `mcp-tool-shell` (Library)
- Implements a `Tool` adapter that runs local shell commands asynchronously
- Defines structures like `ShellCommandTool` implementing `Tool`
- Processes parameters such as { "cmd": "ls", "args": ["-la"] }
- Depends on `mcp-core` and uses std::process::Command or tokio::process

### `mcp-server` (Library)
- Main orchestration crate composing a server from a `Transport` and set of `Tools`
- Provides structures like `MCPServer<T: Transport>` with methods for tool registration and execution
- Optionally integrates JSON-RPC (via `mcp-jsonrpc`) or other protocols for request routing, tool execution, and response handling
- Depends on `mcp-core` and optionally on adapter crates as needed

### `mcp-demo` (Binary, Optional)
- Demonstrates usage with a small executable
- Creates server instances, registers tools, starts async I/O loops, and logs information
- Useful as a reference implementation or for running a simple local MCP server
- Depends on `mcp-server` and chosen adapter implementations

## Architecture Benefits

The modular crate structure provides several advantages:

- `mcp-core` serves as a minimal foundation with clear domain boundaries
- `mcp-jsonrpc` encapsulates protocol-specific logic for JSON-RPC
- Transport crates abstract communication protocols for different mediums
- Tool crates encapsulate specific functionality implementations
- `mcp-server` composes these components into a cohesive system
- `mcp-demo` provides practical usage examples

This separation of concerns allows for flexibility in deployment, easier testing, and the ability to mix and match components as needed.

