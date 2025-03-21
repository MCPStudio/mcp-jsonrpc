use tokio::io::{ReadHalf, WriteHalf}; // pour le split de l'io entre Read et Write

#[cfg(unix)]
use crate::error::helpers;
#[cfg(unix)]
use crate::transport::base::{JsonRpcTransport, Transport};
#[cfg(unix)]
use async_trait::async_trait;
#[cfg(unix)]
use mcp_error::Result as McpResult;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

#[cfg(unix)]
pub struct UnixTransport(JsonRpcTransport<ReadHalf<UnixStream>, WriteHalf<UnixStream>>);



#[cfg(unix)]
impl UnixTransport {
    /// Create a new Unix domain socket transport by connecting to the given path
    pub async fn connect(path: impl AsRef<Path>) -> McpResult<Self> {
        let stream = UnixStream::connect(path)
            .await
            .map_err(|e| helpers::transport_error(&format!("Failed to connect: {}", e)))?;

        Ok(Self(JsonRpcTransport::new(stream)))
    }

    /// Create a Unix domain socket listener that can accept JSON-RPC connections
    pub async fn bind(path: impl AsRef<Path>) -> McpResult<UnixListener> {
        UnixListener::bind(path)
            .map_err(|e| helpers::transport_error(&format!("Failed to bind: {}", e)))
    }
}

#[cfg(unix)]
#[async_trait]
impl Transport for UnixTransport {
    async fn receive(&mut self) -> McpResult<String> {
        self.0.receive().await
    }

    async fn send(&mut self, message: &str) -> McpResult<()> {
        self.0.send(message).await
    }
}

#[cfg(not(unix))]
use crate::error::helpers;
#[cfg(not(unix))]
use mcp_error::Result as McpResult;

#[cfg(not(unix))]
pub struct UnixTransport {}

#[cfg(not(unix))]
impl UnixTransport {
    pub async fn connect<P>(_path: P) -> McpResult<Self> {
        Err(helpers::transport_error(
            "Unix domain sockets are not supported on this platform",
        ))
    }

    pub async fn bind<P>(_path: P) -> McpResult<()> {
        Err(helpers::transport_error(
            "Unix domain sockets are not supported on this platform",
        ))
    }
}
