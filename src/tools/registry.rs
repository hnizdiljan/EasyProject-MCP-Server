use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use tracing::{debug, error, info};

use crate::config::AppConfig;
use crate::api::EasyProjectClient;
use crate::mcp::protocol::{Tool, ToolInputSchema, CallToolResult};

use super::executor::ToolExecutor;
use super::project_tools::*;
use super::issue_tools::*;
use super::user_tools::*;
use super::time_entry_tools::*;
use super::report_tools::*;
use super::milestone_tools::*;
use super::enumeration_tools::*;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolExecutor>>,
}

impl ToolRegistry {
    pub fn new(api_client: EasyProjectClient, config: &AppConfig) -> Self {
        let mut tools: HashMap<String, Arc<dyn ToolExecutor>> = HashMap::new();
        
        info!("Inicializuji MCP tools...");
        
        // Project tools
        if config.tools.projects.enabled {
            let list_projects = Arc::new(ListProjectsTool::new(api_client.clone(), config.clone()));
            let get_project = Arc::new(GetProjectTool::new(api_client.clone(), config.clone()));
            let create_project = Arc::new(CreateProjectTool::new(api_client.clone(), config.clone()));
            let update_project = Arc::new(UpdateProjectTool::new(api_client.clone(), config.clone()));
            let delete_project = Arc::new(DeleteProjectTool::new(api_client.clone(), config.clone()));
            
            tools.insert(list_projects.name().to_string(), list_projects);
            tools.insert(get_project.name().to_string(), get_project);
            tools.insert(create_project.name().to_string(), create_project);
            tools.insert(update_project.name().to_string(), update_project);
            tools.insert(delete_project.name().to_string(), delete_project);
            
            info!("Registrovány project tools");
        }
        
        // Issue tools
        if config.tools.issues.enabled {
            let list_issues = Arc::new(ListIssuesTool::new(api_client.clone(), config.clone()));
            let get_issue = Arc::new(GetIssueTool::new(api_client.clone(), config.clone()));
            let create_issue = Arc::new(CreateIssueTool::new(api_client.clone(), config.clone()));
            let update_issue = Arc::new(UpdateIssueTool::new(api_client.clone(), config.clone()));
            let assign_issue = Arc::new(AssignIssueTool::new(api_client.clone(), config.clone()));
            let complete_issue = Arc::new(CompleteIssueTool::new(api_client.clone(), config.clone()));
            let get_issue_enumerations = Arc::new(GetIssueEnumerationsTool::new(api_client.clone(), config.clone()));

            tools.insert(list_issues.name().to_string(), list_issues);
            tools.insert(get_issue.name().to_string(), get_issue);
            tools.insert(create_issue.name().to_string(), create_issue);
            tools.insert(update_issue.name().to_string(), update_issue);
            tools.insert(assign_issue.name().to_string(), assign_issue);
            tools.insert(complete_issue.name().to_string(), complete_issue);
            tools.insert(get_issue_enumerations.name().to_string(), get_issue_enumerations);

            info!("Registrovány issue tools");
        }
        
        // User tools
        if config.tools.users.enabled {
            let list_users = Arc::new(ListUsersTool::new(api_client.clone(), config.clone()));
            let get_user = Arc::new(GetUserTool::new(api_client.clone(), config.clone()));
            let get_user_workload = Arc::new(GetUserWorkloadTool::new(api_client.clone(), config.clone()));
            
            tools.insert(list_users.name().to_string(), list_users);
            tools.insert(get_user.name().to_string(), get_user);
            tools.insert(get_user_workload.name().to_string(), get_user_workload);
            
            info!("Registrovány user tools");
        }
        
        // Time entry tools
        if config.tools.time_entries.enabled {
            let list_time_entries = Arc::new(ListTimeEntriesTool::new(api_client.clone(), config.clone()));
            let get_time_entry = Arc::new(GetTimeEntryTool::new(api_client.clone(), config.clone()));
            let create_time_entry = Arc::new(CreateTimeEntryTool::new(api_client.clone(), config.clone()));
            let update_time_entry = Arc::new(UpdateTimeEntryTool::new(api_client.clone(), config.clone()));
            let delete_time_entry = Arc::new(DeleteTimeEntryTool::new(api_client.clone(), config.clone()));
            let log_time = Arc::new(LogTimeTool::new(api_client.clone(), config.clone()));
            
            tools.insert(list_time_entries.name().to_string(), list_time_entries);
            tools.insert(get_time_entry.name().to_string(), get_time_entry);
            tools.insert(create_time_entry.name().to_string(), create_time_entry);
            tools.insert(update_time_entry.name().to_string(), update_time_entry);
            tools.insert(delete_time_entry.name().to_string(), delete_time_entry);
            tools.insert(log_time.name().to_string(), log_time);
            
            info!("Registrovány time entry tools");
        }
        
        // Report tools
        if config.tools.reports.enabled {
            let generate_project_report = Arc::new(GenerateProjectReportTool::new(api_client.clone(), config.clone()));
            let get_dashboard_data = Arc::new(GetDashboardDataTool::new(api_client.clone(), config.clone()));
            
            tools.insert(generate_project_report.name().to_string(), generate_project_report);
            tools.insert(get_dashboard_data.name().to_string(), get_dashboard_data);
            
            info!("Registrovány report tools");
        }
        
        // Milestone tools
        if config.tools.milestones.enabled {
            let list_milestones = Arc::new(ListMilestonesTool::new(api_client.clone(), config.clone()));
            let get_milestone = Arc::new(GetMilestoneTool::new(api_client.clone(), config.clone()));
            let create_milestone = Arc::new(CreateMilestoneTool::new(api_client.clone(), config.clone()));
            let update_milestone = Arc::new(UpdateMilestoneTool::new(api_client.clone(), config.clone()));
            let delete_milestone = Arc::new(DeleteMilestoneTool::new(api_client.clone(), config.clone()));
            
            tools.insert(list_milestones.name().to_string(), list_milestones);
            tools.insert(get_milestone.name().to_string(), get_milestone);
            tools.insert(create_milestone.name().to_string(), create_milestone);
            tools.insert(update_milestone.name().to_string(), update_milestone);
            tools.insert(delete_milestone.name().to_string(), delete_milestone);
            
            info!("Registrovány milestone tools");
        }
        
        info!("Celkem registrováno {} tools", tools.len());
        
        Self { tools }
    }
    
    /// Vrátí seznam všech dostupných tools pro MCP protokol
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|tool| Tool {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: Some(tool.input_schema()),
                    required: None,
                    additional_properties: Some(false),
                },
            })
            .collect()
    }
    
    /// Spustí tool s danými argumenty
    pub async fn execute_tool(&self, tool_name: &str, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Spouštím tool: {} s argumenty: {:?}", tool_name, arguments);
        
        match self.tools.get(tool_name) {
            Some(tool) => {
                match tool.execute(arguments).await {
                    Ok(result) => {
                        debug!("Tool {} úspěšně dokončen", tool_name);
                        Ok(result)
                    }
                    Err(e) => {
                        error!("Chyba při spouštění tool {}: {}", tool_name, e);
                        Err(e)
                    }
                }
            }
            None => {
                error!("Tool {} nenalezen", tool_name);
                Err(format!("Tool '{}' nenalezen", tool_name).into())
            }
        }
    }
    
    /// Vrátí počet registrovaných tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
    
    /// Zkontroluje, zda je tool registrován
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tools.contains_key(tool_name)
    }
} 