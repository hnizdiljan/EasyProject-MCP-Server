use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};
use chrono::{DateTime, Utc, NaiveDate, Local};

use crate::api::EasyProjectClient;
use crate::config::AppConfig;
use crate::mcp::protocol::{CallToolResult, ToolResult};
use super::executor::ToolExecutor;

// === GENERATE PROJECT REPORT TOOL ===

pub struct GenerateProjectReportTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl GenerateProjectReportTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct GenerateProjectReportArgs {
    project_id: i32,
    #[serde(default)]
    from_date: Option<String>,
    #[serde(default)]
    to_date: Option<String>,
    #[serde(default)]
    include_time_entries: Option<bool>,
    #[serde(default)]
    include_issues: Option<bool>,
    #[serde(default)]
    include_users: Option<bool>,
}

#[async_trait]
impl ToolExecutor for GenerateProjectReportTool {
    fn name(&self) -> &str {
        "generate_project_report"
    }
    
    fn description(&self) -> &str {
        "Generuje detailní sestavu k projektu včetně statistik úkolů, času a uživatelů"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "project_id": {
                "type": "integer",
                "description": "ID projektu pro generování sestavy (povinné)"
            },
            "from_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum od pro filtrování dat (formát: YYYY-MM-DD)"
            },
            "to_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum do pro filtrování dat (formát: YYYY-MM-DD)"
            },
            "include_time_entries": {
                "type": "boolean",
                "description": "Zahrnout časové záznamy do sestavy (výchozí: true)",
                "default": true
            },
            "include_issues": {
                "type": "boolean",
                "description": "Zahrnout úkoly do sestavy (výchozí: true)",
                "default": true
            },
            "include_users": {
                "type": "boolean",
                "description": "Zahrnout přehled uživatelů do sestavy (výchozí: true)",
                "default": true
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GenerateProjectReportArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'project_id'")?
        )?;
        
        let include_time_entries = args.include_time_entries.unwrap_or(true);
        let include_issues = args.include_issues.unwrap_or(true);
        let include_users = args.include_users.unwrap_or(true);
        
        debug!("Generuji sestavu pro projekt {}", args.project_id);
        
        // 1. Získáme detail projektu
        let project_response = match self.api_client.get_project(args.project_id, Some(vec!["trackers".to_string(), "enabled_modules".to_string()])).await {
            Ok(response) => response,
            Err(e) => {
                error!("Chyba při získávání projektu {}: {}", args.project_id, e);
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání projektu {}: {}", args.project_id, e))
                ]));
            }
        };
        
        let project = &project_response.project;
        let mut report = json!({
            "project": {
                "id": project.id,
                "name": project.name,
                "description": project.description,
                "status": project.status,
                "created_on": project.created_on,
                "updated_on": project.updated_on
            },
            "report_generated_at": Utc::now(),
            "period": {
                "from": args.from_date,
                "to": args.to_date
            }
        });
        
        // 2. Statistiky úkolů (pokud je požadováno)
        if include_issues {
            match self.api_client.list_issues(Some(args.project_id), Some(1000), None, None).await {
                Ok(issues_response) => {
                    let issues = &issues_response.issues;
                    
                    // Filtrování podle data
                    let filtered_issues: Vec<_> = if args.from_date.is_some() || args.to_date.is_some() {
                        issues.iter().filter(|issue| {
                            if let Some(ref created_on) = issue.created_on {
                                let issue_date = created_on.format("%Y-%m-%d").to_string();
                                
                                let after_from = args.from_date.as_ref()
                                    .map(|from| issue_date >= *from)
                                    .unwrap_or(true);
                                    
                                let before_to = args.to_date.as_ref()
                                    .map(|to| issue_date <= *to)
                                    .unwrap_or(true);
                                    
                                after_from && before_to
                            } else {
                                true
                            }
                        }).collect()
                    } else {
                        issues.iter().collect()
                    };
                    
                    let total_issues = filtered_issues.len();
                    let completed_issues = filtered_issues.iter()
                        .filter(|issue| issue.done_ratio.unwrap_or(0) == 100)
                        .count();
                    let in_progress_issues = filtered_issues.iter()
                        .filter(|issue| {
                            let ratio = issue.done_ratio.unwrap_or(0);
                            ratio > 0 && ratio < 100
                        })
                        .count();
                    let pending_issues = total_issues - completed_issues - in_progress_issues;
                    
                    let total_estimated_hours: f64 = filtered_issues.iter()
                        .filter_map(|issue| issue.estimated_hours)
                        .sum();
                    
                    // Seskupení podle statusu
                    let mut status_counts = std::collections::HashMap::new();
                    for issue in &filtered_issues {
                        *status_counts.entry(&issue.status.name).or_insert(0) += 1;
                    }
                    
                    // Seskupení podle priority
                    let mut priority_counts = std::collections::HashMap::new();
                    for issue in &filtered_issues {
                        *priority_counts.entry(&issue.priority.name).or_insert(0) += 1;
                    }
                    
                    report["issues"] = json!({
                        "summary": {
                            "total": total_issues,
                            "completed": completed_issues,
                            "in_progress": in_progress_issues,
                            "pending": pending_issues,
                            "completion_rate": if total_issues > 0 { 
                                (completed_issues as f64 / total_issues as f64 * 100.0).round() 
                            } else { 0.0 },
                            "total_estimated_hours": total_estimated_hours
                        },
                        "by_status": status_counts,
                        "by_priority": priority_counts,
                        "details": filtered_issues
                    });
                }
                Err(e) => {
                    error!("Chyba při získávání úkolů pro projekt {}: {}", args.project_id, e);
                    report["issues"] = json!({"error": format!("Chyba při získávání úkolů: {}", e)});
                }
            }
        }
        
        // 3. Časové záznamy (pokud je požadováno)
        if include_time_entries {
            match self.api_client.list_time_entries(Some(args.project_id), None, Some(1000), None, args.from_date.clone(), args.to_date.clone()).await {
                Ok(time_entries_response) => {
                    let time_entries = &time_entries_response.time_entries;
                    
                    // Filtrování podle data
                    let filtered_entries: Vec<_> = if args.from_date.is_some() || args.to_date.is_some() {
                        time_entries.iter().filter(|entry| {
                            let entry_date = entry.spent_on.format("%Y-%m-%d").to_string();
                            
                            let after_from = args.from_date.as_ref()
                                .map(|from| entry_date >= *from)
                                .unwrap_or(true);
                                
                            let before_to = args.to_date.as_ref()
                                .map(|to| entry_date <= *to)
                                .unwrap_or(true);
                                
                            after_from && before_to
                        }).collect()
                    } else {
                        time_entries.iter().collect()
                    };
                    
                    let total_hours: f64 = filtered_entries.iter()
                        .map(|entry| entry.hours)
                        .sum();
                    
                    // Seskupení podle uživatelů
                    let mut user_hours = std::collections::HashMap::new();
                    for entry in &filtered_entries {
                        let hours = user_hours.entry(&entry.user.name).or_insert(0.0);
                        *hours += entry.hours;
                    }
                    
                    // Seskupení podle aktivit
                    let mut activity_hours = std::collections::HashMap::new();
                    for entry in &filtered_entries {
                        let hours = activity_hours.entry(&entry.activity.name).or_insert(0.0);
                        *hours += entry.hours;
                    }
                    
                    report["time_entries"] = json!({
                        "summary": {
                            "total_entries": filtered_entries.len(),
                            "total_hours": total_hours,
                            "average_per_entry": if !filtered_entries.is_empty() { 
                                total_hours / filtered_entries.len() as f64 
                            } else { 0.0 }
                        },
                        "by_user": user_hours,
                        "by_activity": activity_hours,
                        "details": filtered_entries
                    });
                }
                Err(e) => {
                    error!("Chyba při získávání časových záznamů pro projekt {}: {}", args.project_id, e);
                    report["time_entries"] = json!({"error": format!("Chyba při získávání časových záznamů: {}", e)});
                }
            }
        }
        
        // 4. Přehled uživatelů (pokud je požadováno)
        if include_users {
            // Získáme seznam všech uživatelů a pak filtrujeme ty, kteří pracují na projektu
            match self.api_client.list_users(Some(100), None).await {
                Ok(users_response) => {
                    // V reálné implementaci bychom získali pouze uživatele projektu
                    // Pro demonstraci použijeme všechny uživatele
                    report["users"] = json!({
                        "summary": {
                            "total_users": users_response.users.len()
                        },
                        "details": users_response.users
                    });
                }
                Err(e) => {
                    error!("Chyba při získávání uživatelů: {}", e);
                    report["users"] = json!({"error": format!("Chyba při získávání uživatelů: {}", e)});
                }
            }
        }
        
        let report_json = serde_json::to_string_pretty(&report)?;
        
        info!("Úspěšně vygenerována sestava pro projekt {} ({})", 
              project.name, args.project_id);
        
        Ok(CallToolResult::success(vec![
            ToolResult::text(format!(
                "Sestava pro projekt '{}' (ID: {}):\n\n{}",
                project.name,
                args.project_id,
                report_json
            ))
        ]))
    }
}

// === GET DASHBOARD DATA TOOL ===

pub struct GetDashboardDataTool {
    api_client: EasyProjectClient,
    config: AppConfig,
}

impl GetDashboardDataTool {
    pub fn new(api_client: EasyProjectClient, config: AppConfig) -> Self {
        Self { api_client, config }
    }
}

#[derive(Debug, Deserialize)]
struct GetDashboardDataArgs {
    #[serde(default)]
    project_ids: Option<Vec<i32>>,
    #[serde(default)]
    user_id: Option<i32>,
    #[serde(default)]
    from_date: Option<String>,
    #[serde(default)]
    to_date: Option<String>,
}

#[async_trait]
impl ToolExecutor for GetDashboardDataTool {
    fn name(&self) -> &str {
        "get_dashboard_data"
    }
    
    fn description(&self) -> &str {
        "Získá agregovaná data pro dashboard - přehled projektů, úkolů a časových záznamů"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "project_ids": {
                "type": "array",
                "description": "Seznam ID projektů pro filtrování (nepovinné)",
                "items": {
                    "type": "integer"
                }
            },
            "user_id": {
                "type": "integer",
                "description": "ID uživatele pro filtrování (nepovinné)"
            },
            "from_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum od pro filtrování dat (formát: YYYY-MM-DD)"
            },
            "to_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum do pro filtrování dat (formát: YYYY-MM-DD)"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GetDashboardDataArgs = if let Some(args) = arguments {
            serde_json::from_value(args)?
        } else {
            GetDashboardDataArgs {
                project_ids: None,
                user_id: None,
                from_date: None,
                to_date: None,
            }
        };
        
        debug!("Získávám dashboard data s filtry: {:?}", args);
        
        let mut dashboard = json!({
            "generated_at": Utc::now(),
            "filters": {
                "project_ids": args.project_ids,
                "user_id": args.user_id,
                "from_date": args.from_date,
                "to_date": args.to_date
            }
        });
        
        // 1. Přehled projektů
        match self.api_client.list_projects(Some(100), None, Some(false)).await {
            Ok(projects_response) => {
                let projects = if let Some(ref project_ids) = args.project_ids {
                    projects_response.projects.into_iter()
                        .filter(|p| project_ids.contains(&p.id))
                        .collect()
                } else {
                    projects_response.projects
                };
                
                let active_projects = projects.iter()
                    .filter(|p| matches!(p.status, crate::api::models::ProjectStatus::Active))
                    .count();
                    
                dashboard["projects"] = json!({
                    "total": projects.len(),
                    "active": active_projects,
                    "closed": projects.iter().filter(|p| matches!(p.status, crate::api::models::ProjectStatus::Closed)).count(),
                    "archived": projects.iter().filter(|p| matches!(p.status, crate::api::models::ProjectStatus::Archived)).count(),
                    "details": projects
                });
            }
            Err(e) => {
                error!("Chyba při získávání projektů: {}", e);
                dashboard["projects"] = json!({"error": format!("Chyba při získávání projektů: {}", e)});
            }
        }
        
        // 2. Přehled úkolů
        match self.api_client.list_issues(None, Some(1000), None, None).await {
            Ok(issues_response) => {
                let mut issues = issues_response.issues;
                
                // Filtrování podle projektů
                if let Some(ref project_ids) = args.project_ids {
                    issues.retain(|issue| project_ids.contains(&issue.project.id));
                }
                
                // Filtrování podle uživatele
                if let Some(user_id) = args.user_id {
                    issues.retain(|issue| {
                        issue.assigned_to.as_ref().map(|u| u.id) == Some(user_id)
                    });
                }
                
                // Filtrování podle data
                if args.from_date.is_some() || args.to_date.is_some() {
                    issues.retain(|issue| {
                        if let Some(ref created_on) = issue.created_on {
                            let issue_date = created_on.format("%Y-%m-%d").to_string();
                            
                            let after_from = args.from_date.as_ref()
                                .map(|from| issue_date >= *from)
                                .unwrap_or(true);
                                
                            let before_to = args.to_date.as_ref()
                                .map(|to| issue_date <= *to)
                                .unwrap_or(true);
                                
                            after_from && before_to
                        } else {
                            true
                        }
                    });
                }
                
                let total_issues = issues.len();
                let completed_issues = issues.iter()
                    .filter(|issue| issue.done_ratio.unwrap_or(0) == 100)
                    .count();
                let overdue_issues = issues.iter()
                    .filter(|issue| {
                        if let Some(ref due_date) = issue.due_date {
                            let today = Local::now().date_naive();
                            due_date < &today && issue.done_ratio.unwrap_or(0) < 100
                        } else {
                            false
                        }
                    })
                    .count();
                
                dashboard["issues"] = json!({
                    "total": total_issues,
                    "completed": completed_issues,
                    "in_progress": issues.iter().filter(|issue| {
                        let ratio = issue.done_ratio.unwrap_or(0);
                        ratio > 0 && ratio < 100
                    }).count(),
                    "pending": total_issues - completed_issues,
                    "overdue": overdue_issues,
                    "completion_rate": if total_issues > 0 { 
                        (completed_issues as f64 / total_issues as f64 * 100.0).round() 
                    } else { 0.0 }
                });
            }
            Err(e) => {
                error!("Chyba při získávání úkolů: {}", e);
                dashboard["issues"] = json!({"error": format!("Chyba při získávání úkolů: {}", e)});
            }
        }
        
        // 3. Přehled časových záznamů
        match self.api_client.list_time_entries(None, args.user_id, Some(1000), None, args.from_date.clone(), args.to_date.clone()).await {
            Ok(time_entries_response) => {
                let mut time_entries = time_entries_response.time_entries;
                
                // Filtrování podle projektů
                if let Some(ref project_ids) = args.project_ids {
                    time_entries.retain(|entry| project_ids.contains(&entry.project.id));
                }
                
                // Filtrování podle data
                if args.from_date.is_some() || args.to_date.is_some() {
                    time_entries.retain(|entry| {
                        let entry_date = entry.spent_on.format("%Y-%m-%d").to_string();
                        
                        let after_from = args.from_date.as_ref()
                            .map(|from| entry_date >= *from)
                            .unwrap_or(true);
                            
                        let before_to = args.to_date.as_ref()
                            .map(|to| entry_date <= *to)
                            .unwrap_or(true);
                            
                        after_from && before_to
                    });
                }
                
                let total_hours: f64 = time_entries.iter().map(|entry| entry.hours).sum();
                let total_entries = time_entries.len();
                
                dashboard["time_entries"] = json!({
                    "total_entries": total_entries,
                    "total_hours": total_hours,
                    "average_per_entry": if total_entries > 0 { 
                        total_hours / total_entries as f64 
                    } else { 0.0 }
                });
            }
            Err(e) => {
                error!("Chyba při získávání časových záznamů: {}", e);
                dashboard["time_entries"] = json!({"error": format!("Chyba při získávání časových záznamů: {}", e)});
            }
        }
        
        let dashboard_json = serde_json::to_string_pretty(&dashboard)?;
        
        info!("Úspěšně získána dashboard data");
        
        Ok(CallToolResult::success(vec![
            ToolResult::text(format!(
                "Dashboard data:\n\n{}",
                dashboard_json
            ))
        ]))
    }
} 