use crate::error::helpers;
use async_trait::async_trait;
use mcp_error::Result as McpResult;
use tokio::io::{
    split, AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadHalf, WriteHalf,
};

/// Transport trait for JSON-RPC communication
#[async_trait]
pub trait Transport {
    async fn receive(&mut self) -> McpResult<String>;
    async fn send(&mut self, message: &str) -> McpResult<()>;
}

/// JSON-RPC transport implementation
pub struct JsonRpcTransport<R, W> {
    reader: BufReader<R>,
    writer: W,
}

impl<T> JsonRpcTransport<ReadHalf<T>, WriteHalf<T>>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    /// Crée un nouveau transport en scindant T (par exemple un TcpStream).
    pub fn new(io: T) -> Self {
        let (r, w) = split(io);
        let reader = BufReader::new(r);
        Self { reader, writer: w }
    }
}

#[async_trait]
impl<T> Transport for JsonRpcTransport<ReadHalf<T>, WriteHalf<T>>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    async fn receive(&mut self) -> McpResult<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line).await {
            Ok(0) => Err(helpers::transport_error("Connection closed")),
            Ok(_) => {
                if !line.contains("\"jsonrpc\":\"2.0\"") && !line.contains("\"jsonrpc\": \"2.0\"") {
                    return Err(helpers::protocol_error("Invalid JSON-RPC message"));
                }
                Ok(line)
            }
            Err(e) => Err(helpers::transport_error(&format!("Failed to read: {}", e))),
        }
    }

    async fn send(&mut self, message: &str) -> McpResult<()> {
        let message_with_newline = format!("{}\n", message);
        self.writer
            .write_all(message_with_newline.as_bytes())
            .await
            .map_err(|e| helpers::transport_error(&format!("Failed to send: {}", e)))
    }
}
