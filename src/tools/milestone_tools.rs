use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::api::EasyProjectClient;
use crate::mcp::protocol::{CallToolResult, ToolResult};
use super::executor::ToolExecutor;

// === LIST MILESTONES TOOL ===

pub struct ListMilestonesTool {
    api_client: EasyProjectClient,
}

impl ListMilestonesTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct ListMilestonesArgs {
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    offset: Option<u32>,
    #[serde(default)]
    project_id: Option<i32>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    easy_query_q: Option<String>,
}

#[async_trait]
impl ToolExecutor for ListMilestonesTool {
    fn name(&self) -> &str {
        "list_milestones"
    }
    
    fn description(&self) -> &str {
        "Získá seznam všech milníků (versions) v EasyProject systému s možností filtrování"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "limit": {
                "type": "integer",
                "description": "Maximální počet milníků k vrácení (výchozí: 25, maximum: 100)",
                "minimum": 1,
                "maximum": 100
            },
            "offset": {
                "type": "integer", 
                "description": "Počet milníků k přeskočení pro stránkování",
                "minimum": 0
            },
            "project_id": {
                "type": "integer",
                "description": "ID projektu pro filtrování milníků"
            },
            "status": {
                "type": "string",
                "description": "Status milníku pro filtrování",
                "enum": ["open", "locked", "closed"]
            },
            "easy_query_q": {
                "type": "string",
                "description": "Volný text pro vyhledávání v milnících"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: ListMilestonesArgs = if let Some(args) = arguments {
            serde_json::from_value(args)?
        } else {
            ListMilestonesArgs {
                limit: Some(25),
                offset: None,
                project_id: None,
                status: None,
                easy_query_q: None,
            }
        };
        
        debug!("Získávám seznam milníků s parametry: {:?}", args);
        
        match self.api_client.list_milestones(
            args.limit, 
            args.offset, 
            args.project_id,
            args.status,
            args.easy_query_q
        ).await {
            Ok(response) => {
                let milestones_json = serde_json::to_string_pretty(&response)?;
                info!("Úspěšně získáno {} milníků", response.versions.len());
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Nalezeno {} milníků (celkem: {}):\n\n{}",
                        response.versions.len(),
                        response.total_count.unwrap_or(response.versions.len() as i32),
                        milestones_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání milníků: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání milníků: {}", e))
                ]))
            }
        }
    }
}

// === GET MILESTONE TOOL ===

pub struct GetMilestoneTool {
    api_client: EasyProjectClient,
}

impl GetMilestoneTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct GetMilestoneArgs {
    id: i32,
}

#[async_trait]
impl ToolExecutor for GetMilestoneTool {
    fn name(&self) -> &str {
        "get_milestone"
    }
    
    fn description(&self) -> &str {
        "Získá detail konkrétního milníku podle ID"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID milníku"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GetMilestoneArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'id'")?
        )?;
        
        debug!("Získávám milník s ID: {}", args.id);
        
        match self.api_client.get_milestone(args.id).await {
            Ok(response) => {
                let milestone_json = serde_json::to_string_pretty(&response.version)?;
                info!("Úspěšně získán milník: {}", response.version.name);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Detail milníku '{}':\n\n{}",
                        response.version.name,
                        milestone_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání milníku {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání milníku {}: {}", args.id, e))
                ]))
            }
        }
    }
}

// === CREATE MILESTONE TOOL ===

pub struct CreateMilestoneTool {
    api_client: EasyProjectClient,
}

impl CreateMilestoneTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct CreateMilestoneArgs {
    project_id: i32,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    effective_date: Option<String>,
    #[serde(default)]
    due_date: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    sharing: Option<String>,
    #[serde(default)]
    default_project_version: Option<bool>,
    #[serde(default)]
    easy_external_id: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateMilestoneTool {
    fn name(&self) -> &str {
        "create_milestone"
    }
    
    fn description(&self) -> &str {
        "Vytvoří nový milník v zadaném projektu"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "project_id": {
                "type": "integer",
                "description": "ID projektu, kde se má milník vytvořit"
            },
            "name": {
                "type": "string",
                "description": "Název milníku"
            },
            "description": {
                "type": "string",
                "description": "Popis milníku"
            },
            "effective_date": {
                "type": "string",
                "format": "date",
                "description": "Datum začátku milníku (YYYY-MM-DD)"
            },
            "due_date": {
                "type": "string",
                "format": "date",
                "description": "Datum ukončení milníku (YYYY-MM-DD)"
            },
            "status": {
                "type": "string",
                "description": "Status milníku",
                "enum": ["open", "locked", "closed"]
            },
            "sharing": {
                "type": "string",
                "description": "Nastavení sdílení milníku",
                "enum": ["none", "descendants", "hierarchy", "tree", "system"]
            },
            "default_project_version": {
                "type": "boolean",
                "description": "Zda je toto výchozí verze projektu"
            },
            "easy_external_id": {
                "type": "string",
                "description": "Externí ID pro integraci s jinými systémy"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: CreateMilestoneArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro vytvoření milníku")?
        )?;
        
        debug!("Vytvářím milník s názvem: {}", args.name);
        
        match self.api_client.create_milestone(
            args.project_id,
            args.name,
            args.description,
            args.effective_date,
            args.due_date,
            args.status,
            args.sharing,
            args.default_project_version,
            args.easy_external_id,
        ).await {
            Ok(response) => {
                let milestone_json = serde_json::to_string_pretty(&response.version)?;
                info!("Úspěšně vytvořen milník: {}", response.version.name);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Milník '{}' byl úspěšně vytvořen s ID {}:\n\n{}",
                        response.version.name,
                        response.version.id,
                        milestone_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při vytváření milníku: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při vytváření milníku: {}", e))
                ]))
            }
        }
    }
}

// === UPDATE MILESTONE TOOL ===

pub struct UpdateMilestoneTool {
    api_client: EasyProjectClient,
}

impl UpdateMilestoneTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateMilestoneArgs {
    id: i32,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    effective_date: Option<String>,
    #[serde(default)]
    due_date: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    sharing: Option<String>,
    #[serde(default)]
    default_project_version: Option<bool>,
    #[serde(default)]
    easy_external_id: Option<String>,
}

#[async_trait]
impl ToolExecutor for UpdateMilestoneTool {
    fn name(&self) -> &str {
        "update_milestone"
    }
    
    fn description(&self) -> &str {
        "Aktualizuje existující milník"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID milníku k aktualizaci"
            },
            "name": {
                "type": "string",
                "description": "Nový název milníku"
            },
            "description": {
                "type": "string",
                "description": "Nový popis milníku"
            },
            "effective_date": {
                "type": "string",
                "format": "date",
                "description": "Nové datum začátku milníku (YYYY-MM-DD)"
            },
            "due_date": {
                "type": "string",
                "format": "date",
                "description": "Nové datum ukončení milníku (YYYY-MM-DD)"
            },
            "status": {
                "type": "string",
                "description": "Nový status milníku",
                "enum": ["open", "locked", "closed"]
            },
            "sharing": {
                "type": "string",
                "description": "Nové nastavení sdílení milníku",
                "enum": ["none", "descendants", "hierarchy", "tree", "system"]
            },
            "default_project_version": {
                "type": "boolean",
                "description": "Zda je toto výchozí verze projektu"
            },
            "easy_external_id": {
                "type": "string",
                "description": "Nové externí ID"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: UpdateMilestoneArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro aktualizaci milníku")?
        )?;
        
        debug!("Aktualizuji milník s ID: {}", args.id);
        
        match self.api_client.update_milestone(
            args.id,
            args.name,
            args.description,
            args.effective_date,
            args.due_date,
            args.status,
            args.sharing,
            args.default_project_version,
            args.easy_external_id,
        ).await {
            Ok(response) => {
                let milestone_json = serde_json::to_string_pretty(&response.version)?;
                info!("Úspěšně aktualizován milník: {}", response.version.name);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Milník '{}' byl úspěšně aktualizován:\n\n{}",
                        response.version.name,
                        milestone_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při aktualizaci milníku {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při aktualizaci milníku {}: {}", args.id, e))
                ]))
            }
        }
    }
}

// === DELETE MILESTONE TOOL ===

pub struct DeleteMilestoneTool {
    api_client: EasyProjectClient,
}

impl DeleteMilestoneTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct DeleteMilestoneArgs {
    id: i32,
}

#[async_trait]
impl ToolExecutor for DeleteMilestoneTool {
    fn name(&self) -> &str {
        "delete_milestone"
    }
    
    fn description(&self) -> &str {
        "Smaže existující milník"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID milníku k smazání"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: DeleteMilestoneArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'id'")?
        )?;
        
        debug!("Mažu milník s ID: {}", args.id);
        
        match self.api_client.delete_milestone(args.id).await {
            Ok(_) => {
                info!("Úspěšně smazán milník s ID: {}", args.id);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Milník s ID {} byl úspěšně smazán",
                        args.id
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při mazání milníku {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při mazání milníku {}: {}", args.id, e))
                ]))
            }
        }
    }
} 