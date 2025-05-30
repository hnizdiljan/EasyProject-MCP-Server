use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Chyba transportní vrstvy: {0}")]
    Transport(#[from] TransportError),
    
    #[error("Chyba protokolu: {0}")]
    Protocol(String),
    
    #[error("Neplatná JSON-RPC zpráva: {0}")]
    InvalidMessage(String),
    
    #[error("Neznámá metoda: {0}")]
    UnknownMethod(String),
    
    #[error("Neplatné parametry: {0}")]
    InvalidParams(String),
    
    #[error("Interní chyba serveru: {0}")]
    InternalError(String),
    
    #[error("Tool nenalezen: {0}")]
    ToolNotFound(String),
    
    #[error("Chyba při volání tool: {0}")]
    ToolError(String),
    
    #[error("Seriace/deserializace error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Chyba při čtení ze stdin: {0}")]
    StdinRead(String),
    
    #[error("Chyba při zápisu do stdout: {0}")]
    StdoutWrite(String),
    
    #[error("WebSocket chyba: {0}")]
    WebSocket(String),
    
    #[error("Spojení uzavřeno")]
    ConnectionClosed,
}

/// JSON-RPC 2.0 Error Response podle MCP specifikace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }
    
    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }
    
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: Some(serde_json::json!({ "method": method })),
        }
    }
    
    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(serde_json::json!({ "details": message })),
        }
    }
    
    pub fn internal_error(message: &str) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(serde_json::json!({ "details": message })),
        }
    }
    
    /// Aplikačně specifické chyby (kódy -32000 až -32099)
    pub fn tool_error(message: &str) -> Self {
        Self {
            code: -32000,
            message: "Tool execution error".to_string(),
            data: Some(serde_json::json!({ "details": message })),
        }
    }
    
    pub fn tool_not_found(tool_name: &str) -> Self {
        Self {
            code: -32001,
            message: "Tool not found".to_string(),
            data: Some(serde_json::json!({ "tool": tool_name })),
        }
    }
    
    pub fn api_error(message: &str) -> Self {
        Self {
            code: -32002,
            message: "EasyProject API error".to_string(),
            data: Some(serde_json::json!({ "details": message })),
        }
    }
}

impl From<McpError> for JsonRpcError {
    fn from(error: McpError) -> Self {
        match error {
            McpError::Protocol(_msg) => JsonRpcError::invalid_request(),
            McpError::InvalidMessage(_msg) => JsonRpcError::parse_error(),
            McpError::UnknownMethod(method) => JsonRpcError::method_not_found(&method),
            McpError::InvalidParams(msg) => JsonRpcError::invalid_params(&msg),
            McpError::ToolNotFound(tool) => JsonRpcError::tool_not_found(&tool),
            McpError::ToolError(msg) => JsonRpcError::tool_error(&msg),
            McpError::InternalError(msg) => JsonRpcError::internal_error(&msg),
            McpError::Serialization(err) => JsonRpcError::internal_error(&err.to_string()),
            McpError::Io(err) => JsonRpcError::internal_error(&err.to_string()),
            McpError::Transport(err) => JsonRpcError::internal_error(&err.to_string()),
        }
    }
}

pub type McpResult<T> = Result<T, McpError>; 