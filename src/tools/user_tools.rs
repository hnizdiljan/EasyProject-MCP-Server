use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::api::EasyProjectClient;
use crate::mcp::protocol::{CallToolResult, ToolResult};
use super::executor::ToolExecutor;

// === LIST USERS TOOL ===

pub struct ListUsersTool {
    api_client: EasyProjectClient,
}

impl ListUsersTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct ListUsersArgs {
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    offset: Option<u32>,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    sort: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[async_trait]
impl ToolExecutor for ListUsersTool {
    fn name(&self) -> &str {
        "list_users"
    }

    fn description(&self) -> &str {
        "Získá seznam všech uživatelů v EasyProject systému s možností fulltextového vyhledávání a filtrování. \
        \n\nPoužití: Pro vyhledání uživatelů podle jména nebo emailu použijte parametr 'search'. \
        Pro filtrování podle stavu použijte 'status' (např. 'active' pro aktivní uživatele). \
        \nPříklad: search='Jan Novák' najde všechny uživatele obsahující tento text ve jménu."
    }

    fn input_schema(&self) -> Value {
        json!({
            "limit": {
                "type": "integer",
                "description": "Maximální počet uživatelů k vrácení (výchozí: 25, maximum: 100)",
                "minimum": 1,
                "maximum": 100
            },
            "offset": {
                "type": "integer",
                "description": "Počet uživatelů k přeskočení pro stránkování",
                "minimum": 0
            },
            "search": {
                "type": "string",
                "description": "Fulltextové vyhledávání ve jménech a emailech uživatelů (např. 'Jan Novák' nebo 'jan@firma.cz')"
            },
            "sort": {
                "type": "string",
                "description": "Řazení výsledků (např. 'lastname' nebo 'created_on:desc'). Formát: 'pole' nebo 'pole:desc'"
            },
            "status": {
                "type": "string",
                "description": "Filtrování podle stavu uživatele",
                "enum": ["active", "locked", "registered"]
            }
        })
    }

    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: ListUsersArgs = if let Some(args) = arguments {
            serde_json::from_value(args)?
        } else {
            ListUsersArgs {
                limit: Some(25),
                offset: None,
                search: None,
                sort: None,
                status: None,
            }
        };

        debug!("Získávám seznam uživatelů s parametry: {:?}", args);

        match self.api_client.list_users(args.limit, args.offset, args.search, None, args.sort, args.status).await {
            Ok(response) => {
                let users_json = serde_json::to_string_pretty(&response)?;
                info!("Úspěšně získáno {} uživatelů", response.users.len());
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Nalezeno {} uživatelů (celkem: {}):\n\n{}",
                        response.users.len(),
                        response.total_count.unwrap_or(response.users.len() as i32),
                        users_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání uživatelů: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání uživatelů: {}", e))
                ]))
            }
        }
    }
}

// === GET USER TOOL ===

pub struct GetUserTool {
    api_client: EasyProjectClient,
}

impl GetUserTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct GetUserArgs {
    id: i32,
}

#[async_trait]
impl ToolExecutor for GetUserTool {
    fn name(&self) -> &str {
        "get_user"
    }
    
    fn description(&self) -> &str {
        "Získá detail konkrétního uživatele podle ID"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID uživatele"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GetUserArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'id'")?
        )?;
        
        debug!("Získávám uživatele s ID: {}", args.id);
        
        match self.api_client.get_user(args.id).await {
            Ok(response) => {
                let user_json = serde_json::to_string_pretty(&response.user)?;
                let firstname = response.user.firstname.as_deref().unwrap_or("N/A");
                let lastname = response.user.lastname.as_deref().unwrap_or("N/A");
                info!("Úspěšně získán uživatel: {} {}", firstname, lastname);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Detail uživatele '{} {}':\n\n{}",
                        firstname,
                        lastname,
                        user_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání uživatele {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání uživatele {}: {}", args.id, e))
                ]))
            }
        }
    }
}

// === GET USER WORKLOAD TOOL ===

pub struct GetUserWorkloadTool {
    api_client: EasyProjectClient,
}

impl GetUserWorkloadTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct GetUserWorkloadArgs {
    id: i32,
    #[serde(default)]
    from_date: Option<String>,
    #[serde(default)]
    to_date: Option<String>,
}

#[async_trait]
impl ToolExecutor for GetUserWorkloadTool {
    fn name(&self) -> &str {
        "get_user_workload"
    }
    
    fn description(&self) -> &str {
        "Získá pracovní vytížení uživatele - přehled přiřazených úkolů a odpracovaných hodin"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID uživatele"
            },
            "from_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum od pro filtrování časových záznamů (formát: YYYY-MM-DD)"
            },
            "to_date": {
                "type": "string",
                "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                "description": "Datum do pro filtrování časových záznamů (formát: YYYY-MM-DD)"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GetUserWorkloadArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'id'")?
        )?;
        
        debug!("Získávám pracovní vytížení uživatele s ID: {}", args.id);
        
        // 1. Získáme detail uživatele
        let user_response = match self.api_client.get_user(args.id).await {
            Ok(response) => response,
            Err(e) => {
                error!("Chyba při získávání uživatele {}: {}", args.id, e);
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání uživatele {}: {}", args.id, e))
                ]));
            }
        };
        
        // 2. Získáme přiřazené úkoly uživatele
        let issues_response = match self.api_client.list_issues(None, Some(100), None, None, None, None, None, None, None, None, None).await {
            Ok(response) => response,
            Err(e) => {
                error!("Chyba při získávání úkolů: {}", e);
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání úkolů: {}", e))
                ]));
            }
        };
        
        // Filtrujeme pouze úkoly přiřazené tomuto uživateli
        let assigned_issues: Vec<_> = issues_response.issues.into_iter()
            .filter(|issue| {
                issue.assigned_to.as_ref().map(|u| u.id) == Some(args.id)
            })
            .collect();
        
        // 3. Získáme časové záznamy uživatele
        let time_entries_response = match self.api_client.list_time_entries(None, None, Some(args.id), Some(100), None, args.from_date.clone(), args.to_date.clone()).await {
            Ok(response) => response,
            Err(e) => {
                error!("Chyba při získávání časových záznamů: {}", e);
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání časových záznamů: {}", e))
                ]));
            }
        };
        
        // Filtrujeme časové záznamy podle data pokud je zadáno
        let filtered_time_entries: Vec<_> = if args.from_date.is_some() || args.to_date.is_some() {
            time_entries_response.time_entries.into_iter()
                .filter(|entry| {
                    let entry_date = entry.spent_on.format("%Y-%m-%d").to_string();
                    
                    let after_from = args.from_date.as_ref()
                        .map(|from| entry_date >= *from)
                        .unwrap_or(true);
                        
                    let before_to = args.to_date.as_ref()
                        .map(|to| entry_date <= *to)
                        .unwrap_or(true);
                        
                    after_from && before_to
                })
                .collect()
        } else {
            time_entries_response.time_entries
        };
        
        // 4. Spočítáme statistiky
        let total_assigned_issues = assigned_issues.len();
        let completed_issues = assigned_issues.iter()
            .filter(|issue| issue.done_ratio.unwrap_or(0) == 100)
            .count();
        let in_progress_issues = assigned_issues.iter()
            .filter(|issue| {
                let ratio = issue.done_ratio.unwrap_or(0);
                ratio > 0 && ratio < 100
            })
            .count();
        let pending_issues = assigned_issues.iter()
            .filter(|issue| issue.done_ratio.unwrap_or(0) == 0)
            .count();
            
        let total_hours: f64 = filtered_time_entries.iter()
            .map(|entry| entry.hours)
            .sum();
            
        let total_estimated_hours: f64 = assigned_issues.iter()
            .filter_map(|issue| issue.estimated_hours)
            .sum();
        
        // 5. Sestavíme response
        let firstname = user_response.user.firstname.as_deref().unwrap_or("N/A");
        let lastname = user_response.user.lastname.as_deref().unwrap_or("N/A");
        
        let workload_summary = json!({
            "user": {
                "id": user_response.user.id,
                "name": format!("{} {}", firstname, lastname),
                "email": user_response.user.mail
            },
            "summary": {
                "total_assigned_issues": total_assigned_issues,
                "completed_issues": completed_issues,
                "in_progress_issues": in_progress_issues,
                "pending_issues": pending_issues,
                "completion_rate": if total_assigned_issues > 0 { 
                    (completed_issues as f64 / total_assigned_issues as f64 * 100.0).round() 
                } else { 0.0 },
                "total_logged_hours": total_hours,
                "total_estimated_hours": total_estimated_hours,
                "time_period": {
                    "from": args.from_date,
                    "to": args.to_date
                }
            },
            "assigned_issues": assigned_issues,
            "time_entries": filtered_time_entries
        });
        
        let workload_json = serde_json::to_string_pretty(&workload_summary)?;
        
        info!("Úspěšně získáno pracovní vytížení uživatele {} {}: {} úkolů, {} hodin", 
              firstname, lastname, 
              total_assigned_issues, total_hours);
        
        Ok(CallToolResult::success(vec![
            ToolResult::text(format!(
                "Pracovní vytížení uživatele '{}' ({} {}):\n\n{}",
                user_response.user.mail.unwrap_or_else(|| "N/A".to_string()),
                firstname,
                lastname,
                workload_json
            ))
        ]))
    }
} 