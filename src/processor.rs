use crate::conversion::{
    domain_to_json_rpc_response, json_rpc_to_domain_request, DomainRequest, DomainResponse,
};
use crate::error::helpers;
use crate::protocol::{
    JsonRpcBatch, JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
use crate::transport::Transport;
use async_trait::async_trait;
use mcp_error::{Error as McpError, Result as McpResult, Severity};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Tool trait representing a service that can be invoked by name
/// In a real implementation, this would be imported from mcp-core
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, params: Value) -> McpResult<Value>;
}

/// Registry for storing and retrieving tools
/// In a real implementation, this would be imported from mcp-core
#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(HashMap::new()),
        }
    }

    /// Create a new registry builder for fluent tool registration
    pub fn builder() -> ToolRegistryBuilder {
        ToolRegistryBuilder::new()
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Register a new tool with the given name
    pub fn insert<T: Tool + 'static>(&mut self, name: &str, tool: T) {
        let tools = Arc::get_mut(&mut self.tools).unwrap();
        tools.insert(name.to_string(), Arc::new(tool));
    }
}

/// Builder for creating ToolRegistry instances
pub struct ToolRegistryBuilder {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistryBuilder {
    /// Create a new empty builder
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool with the given name
    pub fn with_tool<T: Tool + 'static>(mut self, name: &str, tool: T) -> Self {
        self.tools.insert(name.to_string(), Arc::new(tool));
        self
    }

    /// Build the final ToolRegistry
    pub fn build(self) -> ToolRegistry {
        ToolRegistry {
            tools: Arc::new(self.tools),
        }
    }
}

/// Simple domain response implementation
struct SimpleDomainResponse {
    id: String,
    result: McpResult<Value>
}

impl DomainResponse for SimpleDomainResponse {
    fn id(&self) -> &str {
        &self.id
    }

    fn result(&self) -> &McpResult<Value> {
        &self.result
    }
}

/// JSON-RPC processor
#[derive(Clone)]
pub struct JsonRpcProcessor<T: Transport> {
    transport: T,
    tool_registry: ToolRegistry,
}

impl<T: Transport> JsonRpcProcessor<T> {
    /// Create a new JSON-RPC processor with the given transport and tool registry
    pub fn new(transport: T, tool_registry: ToolRegistry) -> Self {
        Self {
            transport,
            tool_registry,
        }
    }

    /// Process a single JSON-RPC request
    async fn process_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Validate the request
        if let Err(e) = request.validate() {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: crate::error::error_codes::INVALID_REQUEST,
                    message: "Invalid request".to_string(),
                    data: Some(json!({ "error": e })),
                }),
                id: request.id.clone(),
            };
        }

        // Convert and process request
        let domain_request = match json_rpc_to_domain_request(&request) {
            Ok(req) => req,
            Err(e) => {
                let (code, message) = crate::error::error_to_json_rpc(&e);
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code,
                        message,
                        data: Some(json!({ "error": e.to_string() })),
                    }),
                    id: request.id.clone(),
                };
            }
        };

        // Get and execute tool
        let response = match self.tool_registry.get(domain_request.tool_name()) {
            Some(tool) => {
                let result = tool.execute(domain_request.params().clone()).await;
                let domain_response = SimpleDomainResponse {
                    id: domain_request.id.to_string(),
                    result,
                };
                domain_to_json_rpc_response(&domain_response)
            }
            None => {
                let err = McpError::new(
                    Severity::Error,
                    "TOOL-NOTFOUND",
                    &format!("Method '{}' not found", domain_request.tool_name()),
                );
                let (code, message) = crate::error::error_to_json_rpc(&err);
                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code,
                        message,
                        data: Some(json!({ "error": err.to_string() })),
                    }),
                    id: request.id.clone(),
                })
            }
        };

        // Return response with validation
        match response {
            Ok(resp) => {
                if let Err(e) = resp.validate() {
                    let err = McpError::new(
                        Severity::Critical,
                        "INTERNAL",
                        &format!("Invalid response generated: {}", e),
                    );
                    let (code, message) = crate::error::error_to_json_rpc(&err);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code,
                            message,
                            data: Some(json!({ "error": err.to_string() })),
                        }),
                        id: request.id.clone(),
                    }
                } else {
                    resp
                }
            }
            Err(e) => {
                let (code, message) = crate::error::error_to_json_rpc(&e);
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code,
                        message,
                        data: Some(json!({ "error": e.to_string() })),
                    }),
                    id: request.id.clone(),
                }
            }
        }
    }

    /// Process a notification (no response required)
    async fn process_notification(&self, notification: JsonRpcNotification) -> McpResult<()> {
        // Validate the notification
        if let Err(e) = notification.validate() {
            return Err(helpers::protocol_error(&format!(
                "Invalid notification: {}",
                e
            )));
        }

        // Convert to domain request (reusing existing conversion)
        let request = JsonRpcRequest {
            jsonrpc: notification.jsonrpc,
            method: notification.method,
            params: notification.params,
            id: crate::protocol::JsonRpcId::Null,
        };

        let domain_request = json_rpc_to_domain_request(&request)?;

        // Execute tool if it exists (ignore result since it's a notification)
        if let Some(tool) = self.tool_registry.get(domain_request.tool_name()) {
            if let Err(e) = tool.execute(domain_request.params().clone()).await {
                return Err(helpers::domain_error(
                    McpError::new(Severity::Error, "TOOL-ERROR", &e.to_string()),
                    "Tool execution failed",
                ));
            }
        }

        Ok(())
    }

    /// Process a batch of requests and/or notifications
    async fn process_batch(&self, batch: JsonRpcBatch) -> Vec<JsonRpcResponse> {
        match batch {
            JsonRpcBatch::Requests(requests) => {
                let mut responses = Vec::with_capacity(requests.len());
                for request in requests {
                    responses.push(self.process_request(request).await);
                }
                responses
            }
            JsonRpcBatch::Notifications(notifications) => {
                // Process all notifications but don't return any responses
                for notification in notifications {
                    let _ = self.process_notification(notification).await;
                }
                Vec::new()
            }
        }
    }

    /// Run the processor in a loop, handling incoming messages
    pub async fn run(&mut self) -> McpResult<()> {
        loop {
            let message = match self.transport.receive().await {
                Ok(msg) => msg,
                Err(e) => {
                    if e.to_string().contains("Connection closed") {
                        return Ok(());
                    }
                    return Err(helpers::transport_error(&format!("Transport error: {}", e)));
                }
            };

            // Try parsing as different message types
            let response = match serde_json::from_str::<JsonRpcBatch>(&message) {
                Ok(batch) => {
                    // Process batch
                    let responses = self.process_batch(batch).await;
                    if responses.is_empty() {
                        continue; // No response needed for notification-only batches
                    }
                    serde_json::to_string(&responses).map_err(|e| helpers::json_error(e))?
                }
                Err(_) => {
                    // Try as single request
                    match serde_json::from_str::<JsonRpcRequest>(&message) {
                        Ok(request) => {
                            let response = self.process_request(request).await;
                            serde_json::to_string(&response).map_err(|e| helpers::json_error(e))?
                        }
                        Err(_) => {
                            // Try as notification
                            match serde_json::from_str::<JsonRpcNotification>(&message) {
                                Ok(notification) => {
                                    let _ = self.process_notification(notification).await;
                                    continue; // No response needed for notifications
                                }
                                Err(e) => {
                                    // Invalid JSON or not a valid message
                                    let err = helpers::json_error(e);
                                    let (code, message) = crate::error::error_to_json_rpc(&err);
                                    let error_response = JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        result: None,
                                        error: Some(JsonRpcError {
                                            code,
                                            message,
                                            data: Some(json!({ "error": err.to_string() })),
                                        }),
                                        id: crate::protocol::JsonRpcId::Null,
                                    };
                                    serde_json::to_string(&error_response)
                                        .map_err(|e| helpers::json_error(e))?
                                }
                            }
                        }
                    }
                }
            };

            // Send response if any
            if !response.is_empty() {
                self.transport.send(&response).await.map_err(|e| {
                    helpers::transport_error(&format!("Failed to send response: {}", e))
                })?;
            }
        }
    }
}
