use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::api::{EasyProjectClient, CreateProjectRequest, CreateProject};
use crate::mcp::protocol::{CallToolResult, ToolResult};
use super::executor::ToolExecutor;

// === LIST PROJECTS TOOL ===

pub struct ListProjectsTool {
    api_client: EasyProjectClient,
}

impl ListProjectsTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct ListProjectsArgs {
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    offset: Option<u32>,
    #[serde(default)]
    include_archived: Option<bool>,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    sort: Option<String>,
}

#[async_trait]
impl ToolExecutor for ListProjectsTool {
    fn name(&self) -> &str {
        "list_projects"
    }

    fn description(&self) -> &str {
        "Získá seznam všech projektů v EasyProject systému s možností fulltextového vyhledávání, filtrování a řazení. \
        \n\nPoužití: Pro vyhledání projektů podle názvu nebo identifikátoru použijte parametr 'search'. \
        \nPříklad: search='Webový projekt' najde všechny projekty obsahující tento text v názvu nebo identifikátoru."
    }

    fn input_schema(&self) -> Value {
        json!({
            "limit": {
                "type": "integer",
                "description": "Maximální počet projektů k vrácení (výchozí: 25, maximum: 100)",
                "minimum": 1,
                "maximum": 100
            },
            "offset": {
                "type": "integer",
                "description": "Počet projektů k přeskočení pro stránkování",
                "minimum": 0
            },
            "include_archived": {
                "type": "boolean",
                "description": "Zda zahrnout archivované projekty (výchozí: false)"
            },
            "search": {
                "type": "string",
                "description": "Fulltextové vyhledávání v názvech a identifikátorech projektů (např. 'webový projekt')"
            },
            "sort": {
                "type": "string",
                "description": "Řazení výsledků (např. 'name' nebo 'created_on:desc'). Formát: 'pole' nebo 'pole:desc'"
            }
        })
    }

    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: ListProjectsArgs = if let Some(args) = arguments {
            serde_json::from_value(args)?
        } else {
            ListProjectsArgs {
                limit: Some(25),
                offset: None,
                include_archived: Some(false),
                search: None,
                sort: None,
            }
        };

        debug!("Získávám seznam projektů s parametry: {:?}", args);

        match self.api_client.list_projects(args.limit, args.offset, args.include_archived, args.search, None, args.sort).await {
            Ok(response) => {
                let projects_json = serde_json::to_string_pretty(&response)?;
                info!("Úspěšně získáno {} projektů", response.projects.len());
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Nalezeno {} projektů (celkem: {}):\n\n{}",
                        response.projects.len(),
                        response.total_count.unwrap_or(response.projects.len() as i32),
                        projects_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání projektů: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání projektů: {}", e))
                ]))
            }
        }
    }
}

// === GET PROJECT TOOL ===

pub struct GetProjectTool {
    api_client: EasyProjectClient,
}

impl GetProjectTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct GetProjectArgs {
    id: i32,
    #[serde(default)]
    include: Option<Vec<String>>,
}

#[async_trait]
impl ToolExecutor for GetProjectTool {
    fn name(&self) -> &str {
        "get_project"
    }
    
    fn description(&self) -> &str {
        "Získá detail konkrétního projektu podle ID"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID projektu"
            },
            "include": {
                "type": "array",
                "description": "Dodatečné informace k zahrnutí (trackers, issue_categories, enabled_modules, atd.)",
                "items": {
                    "type": "string",
                    "enum": ["trackers", "issue_categories", "issue_custom_fields", "enabled_modules", "completed_percent", "journals", "easy_stakeholders"]
                }
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GetProjectArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'id'")?
        )?;
        
        debug!("Získávám projekt s ID: {}", args.id);
        
        match self.api_client.get_project(args.id, args.include).await {
            Ok(response) => {
                let project_json = serde_json::to_string_pretty(&response.project)?;
                info!("Úspěšně získán projekt: {}", response.project.name);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Detail projektu '{}':\n\n{}",
                        response.project.name,
                        project_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání projektu {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání projektu {}: {}", args.id, e))
                ]))
            }
        }
    }
}

// === CREATE PROJECT TOOL ===

pub struct CreateProjectTool {
    api_client: EasyProjectClient,
}

impl CreateProjectTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct CreateProjectArgs {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    is_public: Option<bool>,
    #[serde(default)]
    parent_id: Option<i32>,
    #[serde(default)]
    inherit_members: Option<bool>,
    #[serde(default)]
    tracker_ids: Option<Vec<i32>>,
    #[serde(default)]
    enabled_module_names: Option<Vec<String>>,
}

#[async_trait]
impl ToolExecutor for CreateProjectTool {
    fn name(&self) -> &str {
        "create_project"
    }
    
    fn description(&self) -> &str {
        "Vytvoří nový projekt v EasyProject systému"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "name": {
                "type": "string",
                "description": "Název projektu (povinné)"
            },
            "description": {
                "type": "string",
                "description": "Popis projektu"
            },
            "identifier": {
                "type": "string",
                "description": "Unikátní identifikátor projektu"
            },
            "homepage": {
                "type": "string",
                "description": "URL domovské stránky projektu"
            },
            "is_public": {
                "type": "boolean",
                "description": "Zda je projekt veřejný"
            },
            "parent_id": {
                "type": "integer",
                "description": "ID nadřazeného projektu"
            },
            "inherit_members": {
                "type": "boolean",
                "description": "Zda dědit členy z nadřazeného projektu"
            },
            "tracker_ids": {
                "type": "array",
                "description": "Seznam ID trackerů povolených v projektu",
                "items": {
                    "type": "integer"
                }
            },
            "enabled_module_names": {
                "type": "array",
                "description": "Seznam názvů povolených modulů",
                "items": {
                    "type": "string"
                }
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: CreateProjectArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro vytvoření projektu")?
        )?;
        
        debug!("Vytvářím nový projekt: {}", args.name);
        
        let project_data = CreateProjectRequest {
            project: CreateProject {
                name: args.name.clone(),
                description: args.description,
                identifier: args.identifier,
                homepage: args.homepage,
                is_public: args.is_public,
                parent_id: args.parent_id,
                inherit_members: args.inherit_members,
                tracker_ids: args.tracker_ids,
                enabled_module_names: args.enabled_module_names,
            }
        };
        
        match self.api_client.create_project(project_data).await {
            Ok(response) => {
                let project_json = serde_json::to_string_pretty(&response.project)?;
                info!("Úspěšně vytvořen projekt: {} (ID: {})", response.project.name, response.project.id);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Projekt '{}' byl úspěšně vytvořen s ID {}:\n\n{}",
                        response.project.name,
                        response.project.id,
                        project_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při vytváření projektu '{}': {}", args.name, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při vytváření projektu '{}': {}", args.name, e))
                ]))
            }
        }
    }
}

// === UPDATE PROJECT TOOL ===

pub struct UpdateProjectTool {
    api_client: EasyProjectClient,
}

impl UpdateProjectTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateProjectArgs {
    id: i32,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    is_public: Option<bool>,
    #[serde(default)]
    parent_id: Option<i32>,
    #[serde(default)]
    inherit_members: Option<bool>,
    #[serde(default)]
    tracker_ids: Option<Vec<i32>>,
    #[serde(default)]
    enabled_module_names: Option<Vec<String>>,
}

#[async_trait]
impl ToolExecutor for UpdateProjectTool {
    fn name(&self) -> &str {
        "update_project"
    }
    
    fn description(&self) -> &str {
        "Aktualizuje existující projekt v EasyProject systému"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID projektu k aktualizaci (povinné)"
            },
            "name": {
                "type": "string",
                "description": "Nový název projektu"
            },
            "description": {
                "type": "string",
                "description": "Nový popis projektu"
            },
            "identifier": {
                "type": "string",
                "description": "Nový identifikátor projektu"
            },
            "homepage": {
                "type": "string",
                "description": "Nová URL domovské stránky"
            },
            "is_public": {
                "type": "boolean",
                "description": "Zda je projekt veřejný"
            },
            "parent_id": {
                "type": "integer",
                "description": "ID nového nadřazeného projektu"
            },
            "inherit_members": {
                "type": "boolean",
                "description": "Zda dědit členy z nadřazeného projektu"
            },
            "tracker_ids": {
                "type": "array",
                "description": "Seznam ID trackerů povolených v projektu",
                "items": {
                    "type": "integer"
                }
            },
            "enabled_module_names": {
                "type": "array",
                "description": "Seznam názvů povolených modulů",
                "items": {
                    "type": "string"
                }
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: UpdateProjectArgs = serde_json::from_value(
            arguments.ok_or("Chybí argumenty pro aktualizaci projektu")?
        )?;
        
        debug!("Aktualizuji projekt s ID: {}", args.id);
        
        // Nejdříve získáme současný stav projektu
        let current_project = match self.api_client.get_project(args.id, None).await {
            Ok(response) => response.project,
            Err(e) => {
                error!("Chyba při získávání projektu {}: {}", args.id, e);
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání projektu {}: {}", args.id, e))
                ]));
            }
        };
        
        let project_data = CreateProjectRequest {
            project: CreateProject {
                name: args.name.unwrap_or(current_project.name.clone()),
                description: args.description.or(current_project.description),
                identifier: args.identifier.or(current_project.identifier),
                homepage: args.homepage.or(current_project.homepage),
                is_public: args.is_public.or(current_project.is_public),
                parent_id: args.parent_id.or(current_project.parent.map(|p| p.id)),
                inherit_members: args.inherit_members.or(current_project.inherit_members),
                tracker_ids: args.tracker_ids.or(current_project.trackers.map(|t| t.into_iter().map(|tr| tr.id).collect())),
                enabled_module_names: args.enabled_module_names.or(current_project.enabled_modules),
            }
        };
        
        match self.api_client.update_project(args.id, project_data).await {
            Ok(response) => {
                let project_json = serde_json::to_string_pretty(&response.project)?;
                info!("Úspěšně aktualizován projekt: {} (ID: {})", response.project.name, response.project.id);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Projekt '{}' (ID: {}) byl úspěšně aktualizován:\n\n{}",
                        response.project.name,
                        response.project.id,
                        project_json
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při aktualizaci projektu {}: {}", args.id, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při aktualizaci projektu {}: {}", args.id, e))
                ]))
            }
        }
    }
}

// === DELETE PROJECT TOOL ===

pub struct DeleteProjectTool {
    api_client: EasyProjectClient,
}

impl DeleteProjectTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct DeleteProjectArgs {
    id: i32,
}

#[async_trait]
impl ToolExecutor for DeleteProjectTool {
    fn name(&self) -> &str {
        "delete_project"
    }
    
    fn description(&self) -> &str {
        "Smaže projekt z EasyProject systému (POZOR: Tato operace je nevratná!)"
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "id": {
                "type": "integer",
                "description": "ID projektu k smazání (povinné)"
            }
        })
    }
    
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: DeleteProjectArgs = serde_json::from_value(
            arguments.ok_or("Chybí povinný parametr 'id'")?
        )?;
        
        debug!("Mažu projekt s ID: {}", args.id);
        
        // Nejdříve získáme název projektu pro potvrzení
        let project_name = match self.api_client.get_project(args.id, None).await {
            Ok(response) => response.project.name,
            Err(e) => {
                error!("Chyba při získávání projektu {} před smazáním: {}", args.id, e);
                return Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání projektu {} před smazáním: {}", args.id, e))
                ]));
            }
        };
        
        match self.api_client.delete_project(args.id).await {
            Ok(_) => {
                info!("Úspěšně smazán projekt: {} (ID: {})", project_name, args.id);
                
                Ok(CallToolResult::success(vec![
                    ToolResult::text(format!(
                        "Projekt '{}' (ID: {}) byl úspěšně smazán.",
                        project_name,
                        args.id
                    ))
                ]))
            }
            Err(e) => {
                error!("Chyba při mazání projektu {} ({}): {}", args.id, project_name, e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při mazání projektu {} ({}): {}", args.id, project_name, e))
                ]))
            }
        }
    }
} 