use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::error::{JsonRpcError, McpResult};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }
    
    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// MCP Initialize Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub experimental: Option<Value>,
    #[serde(default)]
    pub sampling: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP Initialize Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub logging: Option<Value>,
    pub prompts: Option<PromptsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Tools List Request/Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<bool>,
}

/// Tool Call Request/Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<ToolResult>,
    #[serde(rename = "isError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolResult {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        resource: ResourceReference,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReference {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<LoggingLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

/// MCP Message types
#[derive(Debug, Clone)]
pub enum McpMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcRequest), // Notifications jsou requests bez id
}

impl McpMessage {
    pub fn from_json(json: &str) -> McpResult<Self> {
        let value: Value = serde_json::from_str(json)?;
        
        // Check if it's a response (has 'result' or 'error' field)
        if value.get("result").is_some() || value.get("error").is_some() {
            let response: JsonRpcResponse = serde_json::from_value(value)?;
            return Ok(McpMessage::Response(response));
        }
        
        // Otherwise it's a request
        let request: JsonRpcRequest = serde_json::from_value(value)?;
        
        // Check if it's a notification (no id field)
        if request.id.is_none() {
            Ok(McpMessage::Notification(request))
        } else {
            Ok(McpMessage::Request(request))
        }
    }
    
    pub fn to_json(&self) -> McpResult<String> {
        match self {
            McpMessage::Request(req) | McpMessage::Notification(req) => {
                Ok(serde_json::to_string(req)?)
            }
            McpMessage::Response(resp) => {
                Ok(serde_json::to_string(resp)?)
            }
        }
    }
}

// Helper functions for creating tool schemas
impl ToolInputSchema {
    pub fn object() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            additional_properties: Some(false),
        }
    }
    
    pub fn with_properties(mut self, properties: Value) -> Self {
        self.properties = Some(properties);
        self
    }
    
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }
}

// Helper functions for tool results
impl ToolResult {
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text { text: text.into() }
    }
    
    pub fn image<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self::Image {
            data: data.into(),
            mime_type: mime_type.into(),
        }
    }
}

impl CallToolResult {
    pub fn success(content: Vec<ToolResult>) -> Self {
        Self {
            content,
            is_error: Some(false),
        }
    }
    
    pub fn error(content: Vec<ToolResult>) -> Self {
        Self {
            content,
            is_error: Some(true),
        }
    }
} 