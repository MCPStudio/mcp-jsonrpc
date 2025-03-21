use crate::error::helpers;
use crate::transport::base::{JsonRpcTransport, Transport};
use mcp_error::Result as McpResult;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{ReadHalf, WriteHalf}; // Pour le split si nécessaire

// Le TcpTransport spécifie désormais les deux types : ReadHalf<TcpStream> et WriteHalf<TcpStream>
pub struct TcpTransport(JsonRpcTransport<ReadHalf<TcpStream>, WriteHalf<TcpStream>>);

impl TcpTransport {
    /// Create a new TCP transport by connecting to the given address
    pub async fn connect(addr: impl Into<SocketAddr>) -> McpResult<Self> {
        let stream = TcpStream::connect(addr.into())
            .await
            .map_err(|e| helpers::transport_error(&format!("Failed to connect: {}", e)))?;

        // Le JsonRpcTransport<ReadHalf<TcpStream>, WriteHalf<TcpStream>> est créé
        let transport = JsonRpcTransport::new(stream);

        Ok(Self(transport))
    }

    /// Create a TCP listener that can accept JSON-RPC connections
    pub async fn bind(addr: impl Into<SocketAddr>) -> McpResult<TcpListener> {
        TcpListener::bind(addr.into())
            .await
            .map_err(|e| helpers::transport_error(&format!("Failed to bind: {}", e)))
    }
}

#[async_trait::async_trait]
impl Transport for TcpTransport {
    async fn receive(&mut self) -> McpResult<String> {
        self.0.receive().await
    }

    async fn send(&mut self, message: &str) -> McpResult<()> {
        self.0.send(message).await
    }
}
