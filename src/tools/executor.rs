use async_trait::async_trait;
use serde_json::Value;
use crate::mcp::protocol::CallToolResult;

/// Trait pro implementaci MCP tools
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Název tool
    fn name(&self) -> &str;
    
    /// Popis tool pro MCP klienta
    fn description(&self) -> &str;
    
    /// JSON schema pro input parametry
    fn input_schema(&self) -> Value;
    
    /// Spustí tool s danými argumenty
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>>;
} 