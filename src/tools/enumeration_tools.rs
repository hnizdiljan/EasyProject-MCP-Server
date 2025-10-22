use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::api::EasyProjectClient;
use crate::mcp::protocol::{CallToolResult, ToolResult};
use super::executor::ToolExecutor;

// === GET ISSUE ENUMERATIONS TOOL ===

pub struct GetIssueEnumerationsTool {
    api_client: EasyProjectClient,
}

impl GetIssueEnumerationsTool {
    pub fn new(api_client: EasyProjectClient, _config: crate::config::AppConfig) -> Self {
        Self { api_client }
    }
}

#[derive(Debug, Deserialize)]
struct GetIssueEnumerationsArgs {
    #[serde(default)]
    project_id: Option<i32>,
}

#[async_trait]
impl ToolExecutor for GetIssueEnumerationsTool {
    fn name(&self) -> &str {
        "get_issue_enumerations"
    }

    fn description(&self) -> &str {
        "Získá číselníky (status, priority, tracker) pro použití při filtrování úkolů. \
        \n\nTool INTERNĚ skenuje všechny issues pomocí paginace a vrací pouze kompaktní seznam ID a názvů. \
        Žádné velké datové množiny nejsou vraceny do LLM kontextu. \
        \n\nVyužití: Zavolejte před použitím list_issues s filtry status_id, priority_id nebo tracker_id."
    }

    fn input_schema(&self) -> Value {
        json!({
            "project_id": {
                "type": "integer",
                "description": "Volitelné ID projektu pro získání specifických číselníků tohoto projektu"
            }
        })
    }

    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let args: GetIssueEnumerationsArgs = if let Some(args) = arguments {
            serde_json::from_value(args)?
        } else {
            GetIssueEnumerationsArgs {
                project_id: None,
            }
        };

        debug!("Volání get_issue_enumerations, project_id: {:?}", args.project_id);

        // Voláme metodu API klienta, která INTERNĚ provede paginaci
        match self.api_client.get_issue_enumerations(args.project_id).await {
            Ok(enumerations) => {
                // Vytvoříme kompaktní textový výstup
                let mut result = String::from("Číselníky pro filtrování úkolů:\n\n");

                result.push_str("STAVY (status_id):\n");
                for status in &enumerations.statuses {
                    result.push_str(&format!("  {} = {}\n", status.id, status.name));
                }

                result.push_str("\nPRIORITY (priority_id):\n");
                for priority in &enumerations.priorities {
                    result.push_str(&format!("  {} = {}\n", priority.id, priority.name));
                }

                result.push_str("\nTYPY ÚKOLŮ (tracker_id):\n");
                for tracker in &enumerations.trackers {
                    result.push_str(&format!("  {} = {}\n", tracker.id, tracker.name));
                }

                result.push_str("\nPoužití:\n");
                result.push_str("- Pro filtrování podle statusu: list_issues s parametrem status_id=<ID>\n");
                result.push_str("- Pro filtrování podle priority: list_issues s parametrem priority_id=<ID>\n");
                result.push_str("- Pro filtrování podle typu: list_issues s parametrem tracker_id=<ID>\n");

                info!("Vráceny číselníky: {} statusů, {} priorit, {} trackerů",
                    enumerations.statuses.len(),
                    enumerations.priorities.len(),
                    enumerations.trackers.len());

                Ok(CallToolResult::success(vec![
                    ToolResult::text(result)
                ]))
            }
            Err(e) => {
                error!("Chyba při získávání číselníků: {}", e);
                Ok(CallToolResult::error(vec![
                    ToolResult::text(format!("Chyba při získávání číselníků: {}", e))
                ]))
            }
        }
    }
}
