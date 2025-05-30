use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};
use tracing::{debug, error, info, warn};
use super::error::{TransportError, McpResult};
use super::protocol::McpMessage;

/// Abstraktní trait pro různé transportní vrstvy
#[async_trait]
pub trait Transport {
    async fn receive(&mut self) -> McpResult<McpMessage>;
    async fn send(&mut self, message: McpMessage) -> McpResult<()>;
    async fn close(&mut self) -> McpResult<()>;
}

/// STDIO Transport - komunikace přes standard input/output
pub struct StdioTransport {
    reader: BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
    is_closed: bool,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(stdin()),
            writer: stdout(),
            is_closed: false,
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn receive(&mut self) -> McpResult<McpMessage> {
        if self.is_closed {
            return Err(TransportError::ConnectionClosed.into());
        }
        
        let mut line = String::new();
        match self.reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF reached
                info!("STDIO: EOF dosažen, ukončuji spojení");
                self.is_closed = true;
                Err(TransportError::ConnectionClosed.into())
            }
            Ok(bytes_read) => {
                debug!("STDIO: Přečteno {} bytů", bytes_read);
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    debug!("STDIO: Prázdný řádek, zkouším další");
                    // Prázdný řádek, zkusíme další
                    return self.receive().await;
                }
                
                debug!("STDIO: Přijata zpráva ({} znaků): {}", trimmed.len(), trimmed);
                match McpMessage::from_json(trimmed) {
                    Ok(msg) => Ok(msg),
                    Err(e) => {
                        error!("STDIO: Chyba při parsování JSON: {} | Obsah: '{}'", e, trimmed);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                error!("STDIO: Chyba při čtení: {}", e);
                Err(TransportError::StdinRead(e.to_string()).into())
            }
        }
    }
    
    async fn send(&mut self, message: McpMessage) -> McpResult<()> {
        if self.is_closed {
            return Err(TransportError::ConnectionClosed.into());
        }
        
        let json = message.to_json()?;
        debug!("STDIO: Odesílám zprávu: {}", json);
        
        match self.writer.write_all(format!("{}\n", json).as_bytes()).await {
            Ok(_) => {
                if let Err(e) = self.writer.flush().await {
                    error!("STDIO: Chyba při flush: {}", e);
                    return Err(TransportError::StdoutWrite(e.to_string()).into());
                }
                Ok(())
            }
            Err(e) => {
                error!("STDIO: Chyba při zápisu: {}", e);
                Err(TransportError::StdoutWrite(e.to_string()).into())
            }
        }
    }
    
    async fn close(&mut self) -> McpResult<()> {
        info!("STDIO: Zavírám spojení");
        self.is_closed = true;
        self.writer.flush().await.map_err(|e| TransportError::StdoutWrite(e.to_string()))?;
        Ok(())
    }
}

/// WebSocket Transport - pro budoucí implementaci
pub struct WebSocketTransport {
    // Pro teď prázdná implementace
    _placeholder: (),
}

impl WebSocketTransport {
    pub fn new(_port: u16) -> Self {
        Self {
            _placeholder: (),
        }
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn receive(&mut self) -> McpResult<McpMessage> {
        // TODO: Implementovat WebSocket support
        warn!("WebSocket transport zatím není implementován");
        Err(TransportError::WebSocket("Není implementován".to_string()).into())
    }
    
    async fn send(&mut self, _message: McpMessage) -> McpResult<()> {
        // TODO: Implementovat WebSocket support
        warn!("WebSocket transport zatím není implementován");
        Err(TransportError::WebSocket("Není implementován".to_string()).into())
    }
    
    async fn close(&mut self) -> McpResult<()> {
        // TODO: Implementovat WebSocket support
        info!("WebSocket: Zavírám spojení");
        Ok(())
    }
}

/// Transport Factory pro vytváření správného typu transportu
pub fn create_transport(transport_type: crate::config::TransportType, port: Option<u16>) -> Box<dyn Transport + Send> {
    match transport_type {
        crate::config::TransportType::Stdio => {
            info!("Inicializuji STDIO transport");
            Box::new(StdioTransport::new())
        }
        crate::config::TransportType::Websocket => {
            let port = port.unwrap_or(8080);
            info!("Inicializuji WebSocket transport na portu {}", port);
            Box::new(WebSocketTransport::new(port))
        }
    }
} 