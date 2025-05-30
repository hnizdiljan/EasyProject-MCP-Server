use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};
use chrono::NaiveDate;

use crate::api::{EasyProjectClient, CreateTimeEntryRequest, CreateTimeEntry};
use crate::config::AppConfig;
use crate::mcp::protocol::{CallToolResult, ToolResult};
use super::executor::ToolExecutor;

// === LIST TIME ENTRIES TOOL ===

pub struct ListTimeEntriesTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl ListTimeEntriesTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct ListTimeEntriesArgs {
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    offset: Option<u32>,
    #[serde(default)]
    project_id: Option<i32>,
    #[serde(default)]
    issue_id: Option<i32>,
    #[serde(default)]
    user_id: Option<i32>,
    #[serde(default)]
    from_date: Option<String>,
    #[serde(default)]
    to_date: Option<String>,
}

#[async_trait]
impl ToolExecutor for ListTimeEntriesTool {
    fn name(&self) -> &str {
        "list_time_entries"
    }
    
    fn description(&self) -> &str {
        "Získá seznam časových záznamů s možností filtrování podle projektu, úkolu, uživatele a data"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "limit": {
                "type": "integer",
                "description": "Maximální počet záznamů k vrácení (výchozí: 25, maximum: 100)",
                "minimum": 1,
                "maximum": 100
            },
            "offset": {
                "type": "integer",
                "description": "Počet záznamů k přeskočení pro stránkování",
                "minimum": 0
            },
            "project_id": {
                "type": "integer",
                "description": "ID projektu pro filtrování"
            },
            "issue_id": {
                "type": "integer",
                "description": "ID úkolu pro filtrování"
            },
            "user_id": {
                "type": "integer",
                "description": "ID uživatele pro filtrování"
            },
            "from_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum od (formát: YYYY-MM-DD)"
            },
            "to_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum do (formát: YYYY-MM-DD)"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: ListTimeEntriesArgs = if let Some(args) = arguments {
            serde_json::from_value(args)?
        } else {
            ListTimeEntriesArgs {
                limit: Some(self.config.tools.time_entries.default_limit),
                offset: None,
                project_id: None,
                issue_id: None,
                user_id: None,
                from_date: None,
                to_date: None,
            }
        };
        
        debug!("Získávám časové záznamy s parametry: {:?}", args);
        
        // Validace dat
        if let Some(ref from_str) = args.from_date {
            if NaiveDate::parse_from_str(from_str, "%Y-%m-%d").is_err() {
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Neplatný formát data 'from_date': {}. Očekávaný formát: YYYY-MM-DD", from_str))
                ]));
            }
        }
        
        if let Some(ref to_str) = args.to_date {
            if NaiveDate::parse_from_str(to_str, "%Y-%m-%d").is_err() {
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Neplatný formát data 'to_date': {}. Očekávaný formát: YYYY-MM-DD", to_str))
                ]));
            }
        }
        
        match self.api_client.list_time_entries(
            args.project_id,
            args.user_id,
            args.limit,
            args.offset
        ).await {
            Ok(response) => {
                let time_entries_json = serde_json::to_string_pretty(&response)?;
                let total_hours: f64 = response.time_entries.iter().map(|te| te.hours).sum();
                
                info!("Úspěšně získáno {} časových záznamů (celkem {} hodin)", 
                      response.time_entries.len(), total_hours);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Nalezeno {} časových záznamů (celkem: {}, {} hodin):\n\n{}",
                        response.time_entries.len(),
                        response.total_count.unwrap_or(response.time_entries.len() as i32),
                        total_hours,
                        time_entries_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání časových záznamů: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání časových záznamů: {}", e))
                ]))
            }
        }
    }
}

// === GET TIME ENTRY TOOL ===

pub struct GetTimeEntryTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl GetTimeEntryTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct GetTimeEntryArgs {
    id: i32,
}

#[async_trait]
impl ToolExecutor for GetTimeEntryTool {
    fn name(&self) -> &str {
        "get_time_entry"
    }
    
    fn description(&self) -> &str {
        "Získá detail konkrétního časového záznamu podle ID"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID časového záznamu"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        // Zatím není implementováno v API klientovi
        Ok(CallToolResult::error(vec![
            ToolResult::text("get_time_entry zatím není implementováno".to_string())
        ]))
    }
}

// === CREATE TIME ENTRY TOOL ===

pub struct CreateTimeEntryTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl CreateTimeEntryTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct CreateTimeEntryArgs {
    hours: f64,
    activity_id: i32,
    spent_on: String,
    #[serde(default)]
    issue_id: Option<i32>,
    #[serde(default)]
    project_id: Option<i32>,
    #[serde(default)]
    comments: Option<String>,
}

#[async_trait]
impl ToolExecutor for CreateTimeEntryTool {
    fn name(&self) -> &str {
        "create_time_entry"
    }
    
    fn description(&self) -> &str {
        "Vytvoří nový časový záznam pro projekt nebo úkol"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "hours": {
                "type": "number",
                "description": "Počet odpracovaných hodin",
                "minimum": 0.01,
                "maximum": 24.0
            },
            "activity_id": {
                "type": "integer",
                "description": "ID aktivity"
            },
            "spent_on": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum práce (formát: YYYY-MM-DD)"
            },
            "issue_id": {
                "type": "integer",
                "description": "ID úkolu (alternativně k project_id)"
            },
            "project_id": {
                "type": "integer",
                "description": "ID projektu (alternativně k issue_id)"
            },
            "comments": {
                "type": "string",
                "description": "Komentář k časovému záznamu"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: CreateTimeEntryArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinné parametry")?
        )?;
        
        debug!("Vytvářím časový záznam: {:?}", args);
        
        // Validace
        if args.hours <= 0.0 || args.hours > 24.0 {
            return Ok(CallToolResult::error(vec![
                ToolResult::text("Počet hodin musí být mezi 0.01 a 24.0".to_string())
            ]));
        }
        
        let spent_on = match NaiveDate::parse_from_str(&args.spent_on, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Neplatný formát data 'spent_on': {}. Očekávaný formát: YYYY-MM-DD", args.spent_on))
                ]));
            }
        };
        
        if args.issue_id.is_none() && args.project_id.is_none() {
            return Ok(CallToolResult::error(vec![
                ToolResult::text("Musí být zadán alespoň jeden z parametrů 'issue_id' nebo 'project_id'".to_string())
            ]));
        }
        
        let time_entry = CreateTimeEntry {
            issue_id: args.issue_id,
            project_id: args.project_id,
            spent_on,
            hours: args.hours,
            activity_id: args.activity_id,
            comments: args.comments,
        };
        
        let request = CreateTimeEntryRequest { time_entry };
        
        match self.api_client.create_time_entry(request).await {
            Ok(response) => {
                info!("Úspěšně vytvořen časový záznam s ID: {}", response.time_entry.id);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Časový záznam úspěšně vytvořen s ID: {} ({} hodin na {})",
                        response.time_entry.id,
                        response.time_entry.hours,
                        response.time_entry.spent_on
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při vytváření časového záznamu: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při vytváření časového záznamu: {}", e))
                ]))
            }
        }
    }
}

// === UPDATE TIME ENTRY TOOL ===

pub struct UpdateTimeEntryTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl UpdateTimeEntryTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateTimeEntryArgs {
    id: i32,
    #[serde(default)]
    hours: Option<f64>,
    #[serde(default)]
    activity_id: Option<i32>,
    #[serde(default)]
    spent_on: Option<String>,
    #[serde(default)]
    issue_id: Option<i32>,
    #[serde(default)]
    project_id: Option<i32>,
    #[serde(default)]
    comments: Option<String>,
}

#[async_trait]
impl ToolExecutor for UpdateTimeEntryTool {
    fn name(&self) -> &str {
        "update_time_entry"
    }
    
    fn description(&self) -> &str {
        "Aktualizuje existující časový záznam"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID časového záznamu"
            },
            "hours": {
                "type": "number",
                "description": "Počet odpracovaných hodin",
                "minimum": 0.01,
                "maximum": 24.0
            },
            "activity_id": {
                "type": "integer",
                "description": "ID aktivity"
            },
            "spent_on": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum práce (formát: YYYY-MM-DD)"
            },
            "issue_id": {
                "type": "integer",
                "description": "ID úkolu"
            },
            "project_id": {
                "type": "integer",
                "description": "ID projektu"
            },
            "comments": {
                "type": "string",
                "description": "Komentář k časovému záznamu"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        // Zatím není implementováno v API klientovi
        Ok(CallToolResult::error(vec![
            ToolResult::text("update_time_entry zatím není implementováno".to_string())
        ]))
    }
}

// === DELETE TIME ENTRY TOOL ===

pub struct DeleteTimeEntryTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl DeleteTimeEntryTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct DeleteTimeEntryArgs {
    id: i32,
}

#[async_trait]
impl ToolExecutor for DeleteTimeEntryTool {
    fn name(&self) -> &str {
        "delete_time_entry"
    }
    
    fn description(&self) -> &str {
        "Smaže časový záznam"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID časového záznamu ke smazání"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        // Zatím není implementováno v API klientovi
        Ok(CallToolResult::error(vec![
            ToolResult::text("delete_time_entry zatím není implementováno".to_string())
        ]))
    }
}

// === LOG TIME TOOL (Simplified) ===

pub struct LogTimeTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl LogTimeTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct LogTimeArgs {
    hours: f64,
    activity_id: i32,
    #[serde(default)]
    issue_id: Option<i32>,
    #[serde(default)]
    project_id: Option<i32>,
    #[serde(default)]
    comments: Option<String>,
    #[serde(default)]
    date: Option<String>,
}

#[async_trait]
impl ToolExecutor for LogTimeTool {
    fn name(&self) -> &str {
        "log_time"
    }
    
    fn description(&self) -> &str {
        "Rychle zaloguje čas na projekt nebo úkol (výchozí datum je dnes)"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "hours": {
                "type": "number",
                "description": "Počet odpracovaných hodin",
                "minimum": 0.01,
                "maximum": 24.0
            },
            "activity_id": {
                "type": "integer",
                "description": "ID aktivity"
            },
            "issue_id": {
                "type": "integer",
                "description": "ID úkolu (alternativně k project_id)"
            },
            "project_id": {
                "type": "integer",
                "description": "ID projektu (alternativně k issue_id)"
            },
            "comments": {
                "type": "string",
                "description": "Komentář k časovému záznamu"
            },
            "date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum práce (formát: YYYY-MM-DD, výchozí: dnes)"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: LogTimeArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinné parametry")?
        )?;
        
        debug!("Loguji čas: {:?}", args);
        
        // Validace hodin
        if args.hours <= 0.0 || args.hours > 24.0 {
            return Ok(CallToolResult::error(vec![
                ToolResult::text("Počet hodin musí být mezi 0.01 a 24.0".to_string())
            ]));
        }
        
        // Datum - výchozí je dnes
        let spent_on = if let Some(date_str) = args.date {
            match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => {
                    return Ok(CallToolResult::error(vec![
                        ToolResult::text(format!("Neplatný formát data: {}. Očekávaný formát: YYYY-MM-DD", date_str))
                    ]));
                }
            }
        } else {
            chrono::Utc::now().date_naive()
        };
        
        if args.issue_id.is_none() && args.project_id.is_none() {
            return Ok(CallToolResult::error(vec![
                ToolResult::text("Musí být zadán alespoň jeden z parametrů 'issue_id' nebo 'project_id'".to_string())
            ]));
        }
        
        let time_entry = CreateTimeEntry {
            issue_id: args.issue_id,
            project_id: args.project_id,
            spent_on,
            hours: args.hours,
            activity_id: args.activity_id,
            comments: args.comments,
        };
        
        let request = CreateTimeEntryRequest { time_entry };
        
        match self.api_client.create_time_entry(request).await {
            Ok(response) => {
                info!("Úspěšně zalogován čas: {} hodin", response.time_entry.hours);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "✅ Čas úspěšně zalogován: {} hodin na {} (ID: {})",
                        response.time_entry.hours,
                        response.time_entry.spent_on,
                        response.time_entry.id
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při logování času: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při logování času: {}", e))
                ]))
            }
        }
    }
} 