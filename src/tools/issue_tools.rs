use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};
use chrono::NaiveDate;

use crate::api::{EasyProjectClient, CreateIssueRequest, CreateIssue};
use crate::config::AppConfig;
use crate::mcp::protocol::{CallToolResult, ToolResult};
use super::executor::ToolExecutor;

// === LIST ISSUES TOOL ===

pub struct ListIssuesTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl ListIssuesTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct ListIssuesArgs {
    #[serde(default)]
    project_id: Option<i32>,
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    offset: Option<u32>,
    #[serde(default)]
    include: Option<Vec<String>>,
}

#[async_trait]
impl ToolExecutor for ListIssuesTool {
    fn name(&self) -> &str {
        "list_issues"
    }
    
    fn description(&self) -> &str {
        "Získá seznam úkolů (issues) s možností filtrování podle projektu a stránkování"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "project_id": {
                "type": "integer",
                "description": "ID projektu pro filtrování úkolů"
            },
            "limit": {
                "type": "integer",
                "description": "Maximální počet úkolů k vrácení (výchozí: 25, maximum: 100)",
                "minimum": 1,
                "maximum": 100
            },
            "offset": {
                "type": "integer",
                "description": "Počet úkolů k přeskočení pro stránkování",
                "minimum": 0
            },
            "include": {
                "type": "array",
                "description": "Dodatečné informace k zahrnutí",
                "items": {
                    "type": "string",
                    "enum": ["attachments", "relations", "total_estimated_time", "spent_time", "checklists"]
                }
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: ListIssuesArgs = if let Some(args) = arguments {
            serde_json::from_value(args)?
        } else {
            ListIssuesArgs {
                project_id: None,
                limit: Some(self.config.tools.issues.default_limit),
                offset: None,
                include: None,
            }
        };
        
        debug!("Získávám seznam úkolů s parametry: {:?}", args);
        
        match self.api_client.list_issues(args.project_id, args.limit, args.offset, args.include).await {
            Ok(response) => {
                let issues_json = serde_json::to_string_pretty(&response)?;
                info!("Úspěšně získáno {} úkolů", response.issues.len());
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Nalezeno {} úkolů (celkem: {}):\n\n{}",
                        response.issues.len(),
                        response.total_count.unwrap_or(response.issues.len() as i32),
                        issues_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání úkolů: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání úkolů: {}", e))
                ]))
            }
        }
    }
}

// === GET ISSUE TOOL ===

pub struct GetIssueTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl GetIssueTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct GetIssueArgs {
    id: i32,
    #[serde(default)]
    include: Option<Vec<String>>,
}

#[async_trait]
impl ToolExecutor for GetIssueTool {
    fn name(&self) -> &str {
        "get_issue"
    }
    
    fn description(&self) -> &str {
        "Získá detail konkrétního úkolu podle ID"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID úkolu"
            },
            "include": {
                "type": "array",
                "description": "Dodatečné informace k zahrnutí",
                "items": {
                    "type": "string",
                    "enum": ["attachments", "relations", "total_estimated_time", "spent_time", "checklists"]
                }
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GetIssueArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'id'")?
        )?;
        
        debug!("Získávám úkol s ID: {}", args.id);
        
        match self.api_client.get_issue(args.id, args.include).await {
            Ok(response) => {
                let issue_json = serde_json::to_string_pretty(&response.issue)?;
                info!("Úspěšně získán úkol: {}", response.issue.subject);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Detail úkolu '{}':\n\n{}",
                        response.issue.subject,
                        issue_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání úkolu {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání úkolu {}: {}", args.id, e))
                ]))
            }
        }
    }
}

// === CREATE ISSUE TOOL ===

pub struct CreateIssueTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl CreateIssueTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct CreateIssueArgs {
    project_id: i32,
    tracker_id: i32,
    status_id: i32,
    priority_id: i32,
    subject: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    category_id: Option<i32>,
    #[serde(default)]
    fixed_version_id: Option<i32>,
    #[serde(default)]
    assigned_to_id: Option<i32>,
    #[serde(default)]
    parent_issue_id: Option<i32>,
    #[serde(default)]
    estimated_hours: Option<f64>,
    #[serde(default)]
    start_date: Option<NaiveDate>,
    #[serde(default)]
    due_date: Option<NaiveDate>,
    #[serde(default)]
    done_ratio: Option<i32>,
}

#[async_trait]
impl ToolExecutor for CreateIssueTool {
    fn name(&self) -> &str {
        "create_issue"
    }
    
    fn description(&self) -> &str {
        "Vytvoří nový úkol v EasyProject systému"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "project_id": {
                "type": "integer",
                "description": "ID projektu (povinné)"
            },
            "tracker_id": {
                "type": "integer",
                "description": "ID trackeru (povinné)"
            },
            "status_id": {
                "type": "integer",
                "description": "ID statusu (povinné)"
            },
            "priority_id": {
                "type": "integer",
                "description": "ID priority (povinné)"
            },
            "subject": {
                "type": "string",
                "description": "Název úkolu (povinné)"
            },
            "description": {
                "type": "string",
                "description": "Popis úkolu"
            },
            "category_id": {
                "type": "integer",
                "description": "ID kategorie"
            },
            "fixed_version_id": {
                "type": "integer",
                "description": "ID verze/milníku"
            },
            "assigned_to_id": {
                "type": "integer",
                "description": "ID uživatele, kterému je úkol přiřazen"
            },
            "parent_issue_id": {
                "type": "integer",
                "description": "ID nadřazeného úkolu"
            },
            "estimated_hours": {
                "type": "number",
                "description": "Odhadované hodiny"
            },
            "start_date": {
                "type": "string",
                "format": "date",
                "description": "Datum zahájení (YYYY-MM-DD)"
            },
            "due_date": {
                "type": "string",
                "format": "date",
                "description": "Termín dokončení (YYYY-MM-DD)"
            },
            "done_ratio": {
                "type": "integer",
                "description": "Procento dokončení (0-100)",
                "minimum": 0,
                "maximum": 100
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: CreateIssueArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro vytvoření úkolu")?
        )?;
        
        debug!("Vytvářím nový úkol: {}", args.subject);
        
        let issue_data = CreateIssueRequest {
            issue: CreateIssue {
                project_id: args.project_id,
                tracker_id: args.tracker_id,
                status_id: args.status_id,
                priority_id: args.priority_id,
                subject: args.subject.clone(),
                description: args.description,
                category_id: args.category_id,
                fixed_version_id: args.fixed_version_id,
                assigned_to_id: args.assigned_to_id,
                parent_issue_id: args.parent_issue_id,
                estimated_hours: args.estimated_hours,
                start_date: args.start_date,
                due_date: args.due_date,
                done_ratio: args.done_ratio,
            }
        };
        
        match self.api_client.create_issue(issue_data).await {
            Ok(response) => {
                let issue_json = serde_json::to_string_pretty(&response.issue)?;
                info!("Úspěšně vytvořen úkol: {} (ID: {})", response.issue.subject, response.issue.id);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Úkol '{}' byl úspěšně vytvořen s ID {}:\n\n{}",
                        response.issue.subject,
                        response.issue.id,
                        issue_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při vytváření úkolu '{}': {}", args.subject, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při vytváření úkolu '{}': {}", args.subject, e))
                ]))
            }
        }
    }
}

// === UPDATE ISSUE TOOL ===

pub struct UpdateIssueTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl UpdateIssueTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateIssueArgs {
    id: i32,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    status_id: Option<i32>,
    #[serde(default)]
    priority_id: Option<i32>,
    #[serde(default)]
    assigned_to_id: Option<i32>,
    #[serde(default)]
    done_ratio: Option<i32>,
    #[serde(default)]
    estimated_hours: Option<f64>,
    #[serde(default)]
    start_date: Option<NaiveDate>,
    #[serde(default)]
    due_date: Option<NaiveDate>,
}

#[async_trait]
impl ToolExecutor for UpdateIssueTool {
    fn name(&self) -> &str {
        "update_issue"
    }
    
    fn description(&self) -> &str {
        "Aktualizuje existující úkol v EasyProject systému"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID úkolu k aktualizaci (povinné)"
            },
            "subject": {
                "type": "string",
                "description": "Nový název úkolu"
            },
            "description": {
                "type": "string",
                "description": "Nový popis úkolu"
            },
            "status_id": {
                "type": "integer",
                "description": "Nové ID statusu"
            },
            "priority_id": {
                "type": "integer",
                "description": "Nové ID priority"
            },
            "assigned_to_id": {
                "type": "integer",
                "description": "ID uživatele, kterému přiřadit úkol"
            },
            "done_ratio": {
                "type": "integer",
                "description": "Nové procento dokončení (0-100)",
                "minimum": 0,
                "maximum": 100
            },
            "estimated_hours": {
                "type": "number",
                "description": "Nové odhadované hodiny"
            },
            "start_date": {
                "type": "string",
                "format": "date",
                "description": "Nové datum zahájení (YYYY-MM-DD)"
            },
            "due_date": {
                "type": "string",
                "format": "date",
                "description": "Nový termín dokončení (YYYY-MM-DD)"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: UpdateIssueArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro aktualizaci úkolu")?
        )?;
        
        debug!("Aktualizuji úkol s ID: {}", args.id);
        
        // Nejdříve získáme současný stav úkolu
        let current_issue = match self.api_client.get_issue(args.id, None).await {
            Ok(response) => response.issue,
            Err(e) => {
                error!("Chyba při získávání úkolu {}: {}", args.id, e);
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání úkolu {}: {}", args.id, e))
                ]));
            }
        };
        
        let issue_data = CreateIssueRequest {
            issue: CreateIssue {
                project_id: current_issue.project.id,
                tracker_id: current_issue.tracker.id,
                status_id: args.status_id.unwrap_or(current_issue.status.id),
                priority_id: args.priority_id.unwrap_or(current_issue.priority.id),
                subject: args.subject.unwrap_or(current_issue.subject.clone()),
                description: args.description.or(current_issue.description),
                category_id: current_issue.category.map(|c| c.id),
                fixed_version_id: current_issue.fixed_version.map(|v| v.id),
                assigned_to_id: args.assigned_to_id.or(current_issue.assigned_to.map(|u| u.id)),
                parent_issue_id: current_issue.parent.map(|p| p.id),
                estimated_hours: args.estimated_hours.or(current_issue.estimated_hours),
                start_date: args.start_date.or(current_issue.start_date),
                due_date: args.due_date.or(current_issue.due_date),
                done_ratio: args.done_ratio.or(current_issue.done_ratio),
            }
        };
        
        match self.api_client.update_issue(args.id, issue_data).await {
            Ok(response) => {
                let issue_json = serde_json::to_string_pretty(&response.issue)?;
                info!("Úspěšně aktualizován úkol: {} (ID: {})", response.issue.subject, response.issue.id);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Úkol '{}' (ID: {}) byl úspěšně aktualizován:\n\n{}",
                        response.issue.subject,
                        response.issue.id,
                        issue_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při aktualizaci úkolu {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při aktualizaci úkolu {}: {}", args.id, e))
                ]))
            }
        }
    }
}

// === ASSIGN ISSUE TOOL ===

pub struct AssignIssueTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl AssignIssueTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct AssignIssueArgs {
    id: i32,
    assigned_to_id: i32,
}

#[async_trait]
impl ToolExecutor for AssignIssueTool {
    fn name(&self) -> &str {
        "assign_issue"
    }
    
    fn description(&self) -> &str {
        "Přiřadí úkol konkrétnímu uživateli"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID úkolu k přiřazení (povinné)"
            },
            "assigned_to_id": {
                "type": "integer",
                "description": "ID uživatele, kterému přiřadit úkol (povinné)"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: AssignIssueArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro přiřazení úkolu")?
        )?;
        
        debug!("Přiřazuji úkol {} uživateli {}", args.id, args.assigned_to_id);
        
        // Použijeme update_issue s pouze změnou assigned_to_id
        let update_args = UpdateIssueArgs {
            id: args.id,
            assigned_to_id: Some(args.assigned_to_id),
            subject: None,
            description: None,
            status_id: None,
            priority_id: None,
            done_ratio: None,
            estimated_hours: None,
            start_date: None,
            due_date: None,
        };
        
        // Delegujeme na UpdateIssueTool
        let update_tool = UpdateIssueTool::new(self.api_client.clone(), self.config.clone());
        let result = update_tool.execute(Some(serde_json::to_value(update_args)?)).await?;
        
        // Upravíme zprávu pro lepší kontext
        match result.is_error {
            Some(true) => Ok(result),
            _ => {
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Úkol {} byl úspěšně přiřazen uživateli {}.",
                        args.id,
                        args.assigned_to_id
                    ))
                ]))
            }
        }
    }
}

// === COMPLETE ISSUE TOOL ===

pub struct CompleteIssueTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl CompleteIssueTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct CompleteIssueArgs {
    id: i32,
    #[serde(default = "default_done_ratio")]
    done_ratio: i32,
}

fn default_done_ratio() -> i32 {
    100
}

#[async_trait]
impl ToolExecutor for CompleteIssueTool {
    fn name(&self) -> &str {
        "complete_task"
    }
    
    fn description(&self) -> &str {
        "Označí úkol jako dokončený (nastaví done_ratio na 100%)"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID úkolu k označení jako dokončený (povinné)"
            },
            "done_ratio": {
                "type": "integer",
                "description": "Procento dokončení (výchozí: 100)",
                "minimum": 0,
                "maximum": 100,
                "default": 100
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: CompleteIssueArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro dokončení úkolu")?
        )?;
        
        debug!("Označuji úkol {} jako dokončený ({}%)", args.id, args.done_ratio);
        
        // Použijeme update_issue s pouze změnou done_ratio
        let update_args = UpdateIssueArgs {
            id: args.id,
            done_ratio: Some(args.done_ratio),
            assigned_to_id: None,
            subject: None,
            description: None,
            status_id: None,
            priority_id: None,
            estimated_hours: None,
            start_date: None,
            due_date: None,
        };
        
        // Delegujeme na UpdateIssueTool
        let update_tool = UpdateIssueTool::new(self.api_client.clone(), self.config.clone());
        let result = update_tool.execute(Some(serde_json::to_value(update_args)?)).await?;
        
        // Upravíme zprávu pro lepší kontext
        match result.is_error {
            Some(true) => Ok(result),
            _ => {
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Úkol {} byl úspěšně označen jako dokončený ({}%).",
                        args.id,
                        args.done_ratio
                    ))
                ]))
            }
        }
    }
} 