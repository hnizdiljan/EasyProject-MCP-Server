use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;
use crate::api::EasyProjectClient;
use crate::tools::ToolRegistry;

use super::error::{McpError, McpResult};
use super::protocol::{*, PromptsCapability, ResourcesCapability};
use super::transport::{Transport, create_transport};

pub struct McpServer {
    config: AppConfig,
    transport: Box<dyn Transport + Send>,
    tool_registry: ToolRegistry,
    is_initialized: bool,
    client_info: Option<ClientInfo>,
}

impl McpServer {
    pub async fn new(config: AppConfig) -> McpResult<Self> {
        info!("Inicializuji MCP Server");
        
        // Vytvoření transportní vrstvy
        let transport = create_transport(
            config.server.transport.clone(),
            config.server.websocket_port
        );
        
        // Vytvoření API klienta
        let api_client = EasyProjectClient::new(&config).await
            .map_err(|e| McpError::InternalError(format!("Nepodařilo se vytvořit API klient: {}", e)))?;
        
        // Inicializace tool registry
        let tool_registry = ToolRegistry::new(api_client, &config);
        
        Ok(Self {
            config,
            transport,
            tool_registry,
            is_initialized: false,
            client_info: None,
        })
    }
    
    pub async fn run(&mut self) -> McpResult<()> {
        info!("MCP Server spuštěn a čeká na zprávy");
        
        loop {
            match self.transport.receive().await {
                Ok(message) => {
                    if let Err(e) = self.handle_message(message).await {
                        error!("Chyba při zpracování zprávy: {}", e);
                        // Pokračujeme v běhu i při chybách
                    }
                }
                Err(McpError::Transport(crate::mcp::error::TransportError::ConnectionClosed)) => {
                    info!("Spojení ukončeno, zastavuji server");
                    break;
                }
                Err(e) => {
                    error!("Chyba transportní vrstvy: {}", e);
                    // Můžeme se rozhodnout, zda pokračovat nebo ukončit
                    break;
                }
            }
        }
        
        // Cleanup
        self.transport.close().await?;
        info!("MCP Server ukončen");
        Ok(())
    }
    
    async fn handle_message(&mut self, message: McpMessage) -> McpResult<()> {
        match message {
            McpMessage::Request(request) => {
                debug!("Zpracovávám request: {}", request.method);
                let response = self.handle_request(request).await;
                self.transport.send(McpMessage::Response(response)).await?;
            }
            McpMessage::Notification(notification) => {
                debug!("Zpracovávám notification: {}", notification.method);
                self.handle_notification(notification).await?;
            }
            McpMessage::Response(_) => {
                warn!("Přijata neočekávaná response zpráva");
            }
        }
        Ok(())
    }
    
    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_tools_list(request.params).await,
            "tools/call" => self.handle_tools_call(request.params).await,
            method => {
                error!("Neznámá metoda: {}", method);
                Err(McpError::UnknownMethod(method.to_string()))
            }
        };
        
        match result {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(error) => JsonRpcResponse::error(request.id, error.into()),
        }
    }
    
    async fn handle_notification(&mut self, notification: JsonRpcRequest) -> McpResult<()> {
        match notification.method.as_str() {
            "notifications/initialized" => {
                info!("Klient potvrdil inicializaci");
                Ok(())
            }
            "notifications/cancelled" => {
                debug!("Operace zrušena");
                Ok(())
            }
            method => {
                warn!("Neznámá notification: {}", method);
                Ok(())
            }
        }
    }
    
    async fn handle_initialize(&mut self, params: Option<Value>) -> McpResult<Value> {
        let params: InitializeParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::InvalidParams(format!("Neplatné parametry initialize: {}", e)))?,
            None => return Err(McpError::InvalidParams("Chybí parametry pro initialize".to_string())),
        };
        
        info!("Inicializace od klienta: {} v{}", params.client_info.name, params.client_info.version);
        
        if params.protocol_version != "2024-11-05" {
            warn!("Nepodporovaná verze MCP protokolu: {}", params.protocol_version);
        }
        
        self.client_info = Some(params.client_info);
        self.is_initialized = true;
        
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                logging: Some(json!({})),
                prompts: Some(PromptsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: self.config.server.name.clone(),
                version: self.config.server.version.clone(),
            },
            instructions: Some("EasyProject MCP Server pro správu projektů, úkolů a uživatelů prostřednictvím EasyProject API.".to_string()),
        };
        
        Ok(serde_json::to_value(result)?)
    }
    
    async fn handle_tools_list(&self, params: Option<Value>) -> McpResult<Value> {
        if !self.is_initialized {
            return Err(McpError::Protocol("Server není inicializován".to_string()));
        }
        
        let _params: ListToolsParams = match params {
            Some(p) => serde_json::from_value(p).unwrap_or_default(),
            None => ListToolsParams { cursor: None },
        };
        
        debug!("Generuji seznam dostupných tools");
        let tools = self.tool_registry.list_tools();
        
        let result = ListToolsResult {
            tools,
            next_cursor: None, // Pro jednoduchost zatím nepodporujeme stránkování
        };
        
        Ok(serde_json::to_value(result)?)
    }
    
    async fn handle_tools_call(&self, params: Option<Value>) -> McpResult<Value> {
        if !self.is_initialized {
            return Err(McpError::Protocol("Server není inicializován".to_string()));
        }
        
        let params: CallToolParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::InvalidParams(format!("Neplatné parametry pro tools/call: {}", e)))?,
            None => return Err(McpError::InvalidParams("Chybí parametry pro tools/call".to_string())),
        };
        
        info!("Volám tool: {}", params.name);
        debug!("Argumenty: {:?}", params.arguments);
        
        let result = self.tool_registry.execute_tool(&params.name, params.arguments).await
            .map_err(|e| {
                error!("Chyba při volání tool {}: {}", params.name, e);
                McpError::ToolError(e.to_string())
            })?;
        
        Ok(serde_json::to_value(result)?)
    }
}

// Default implementace pro ListToolsParams
impl Default for ListToolsParams {
    fn default() -> Self {
        Self { cursor: None }
    }
} 