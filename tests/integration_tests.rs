use easyproject_mcp_server::config::AppConfig;
use easyproject_mcp_server::mcp::McpServer;
use easyproject_mcp_server::tools::ToolRegistry;
use easyproject_mcp_server::api::EasyProjectClient;
use tokio_test;
use serde_json::json;

#[tokio::test]
async fn test_config_loading() {
    let config = AppConfig::default();
    
    assert_eq!(config.server.name, "EasyProject MCP Server");
    assert_eq!(config.server.version, "1.0.0");
    assert!(config.tools.projects.enabled);
    assert!(config.tools.issues.enabled);
    assert!(config.tools.users.enabled);
    assert!(config.tools.time_entries.enabled);
    assert!(config.tools.reports.enabled);
}

#[tokio::test]
async fn test_config_validation() {
    let mut config = AppConfig::default();
    
    // Platná konfigurace by měla projít
    config.easyproject.api_key = Some("test-key".to_string());
    assert!(config.validate().is_ok());
    
    // Neplatná URL by měla selhat
    config.easyproject.base_url = "not-a-url".to_string();
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_tool_registry_initialization() {
    let config = AppConfig::default();
    let client = create_mock_client(&config).await;
    
    let registry = ToolRegistry::new(client, &config);
    
    // Zkontrolujeme, že jsou registrovány základní nástroje
    assert!(registry.has_tool("list_projects"));
    assert!(registry.has_tool("get_project"));
    assert!(registry.has_tool("create_project"));
    
    assert!(registry.has_tool("list_issues"));
    assert!(registry.has_tool("get_issue"));
    assert!(registry.has_tool("create_issue"));
    assert!(registry.has_tool("assign_issue"));
    assert!(registry.has_tool("complete_task"));
    
    assert!(registry.has_tool("list_users"));
    assert!(registry.has_tool("get_user"));
    assert!(registry.has_tool("get_user_workload"));
    
    assert!(registry.has_tool("list_time_entries"));
    assert!(registry.has_tool("log_time"));
    assert!(registry.has_tool("update_time_entry"));
    
    assert!(registry.has_tool("generate_project_report"));
    assert!(registry.has_tool("get_dashboard_data"));
    
    // Zkontrolujeme celkový počet nástrojů
    assert!(registry.tool_count() > 10);
}

#[tokio::test]
async fn test_tool_list_generation() {
    let config = AppConfig::default();
    let client = create_mock_client(&config).await;
    let registry = ToolRegistry::new(client, &config);
    
    let tools = registry.list_tools();
    
    // Zkontrolujeme, že seznam obsahuje všechny nástroje
    assert!(!tools.is_empty());
    
    // Zkontrolujeme, že každý nástroj má správnou strukturu
    for tool in tools {
        assert!(!tool.name.is_empty());
        assert!(!tool.description.is_empty());
        assert_eq!(tool.input_schema.schema_type, "object");
    }
}

#[tokio::test]
async fn test_invalid_tool_execution() {
    let config = AppConfig::default();
    let client = create_mock_client(&config).await;
    let registry = ToolRegistry::new(client, &config);
    
    // Pokus o spuštění neexistujícího nástroje
    let result = registry.execute_tool("nonexistent_tool", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_execution_without_required_args() {
    let config = AppConfig::default();
    let client = create_mock_client(&config).await;
    let registry = ToolRegistry::new(client, &config);
    
    // Pokus o spuštění nástroje bez povinných argumentů
    let result = registry.execute_tool("get_project", None).await;
    assert!(result.is_ok());
    
    // Výsledek by měl obsahovat chybu
    let call_result = result.unwrap();
    assert_eq!(call_result.is_error, Some(true));
}

// Pomocná funkce pro vytvoření mock klienta
async fn create_mock_client(config: &AppConfig) -> EasyProjectClient {
    // V reálných testech bychom použili mock server
    // Pro teď vytvoříme klienta s falešnou konfigurací
    let mut test_config = config.clone();
    test_config.easyproject.base_url = "http://localhost:8080".to_string();
    test_config.easyproject.api_key = Some("test-key".to_string());
    
    // Pozor: toto selže, ale pro účely testů struktury je to OK
    match EasyProjectClient::new(&test_config).await {
        Ok(client) => client,
        Err(_) => {
            // Fallback pro případy, kde nemůžeme vytvořit skutečný klient
            panic!("Pro integration testy je potřeba mock server nebo skutečné API")
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use easyproject_mcp_server::utils::validation::*;
    use easyproject_mcp_server::utils::date_utils::*;
    use easyproject_mcp_server::utils::formatting::*;
    use chrono::{NaiveDate, DateTime, Utc};

    #[test]
    fn test_date_validation() {
        assert!(is_valid_date_string("2023-12-25"));
        assert!(!is_valid_date_string("2023-13-25"));
        assert!(!is_valid_date_string("not-a-date"));
        assert!(!is_valid_date_string("2023/12/25"));
    }

    #[test]
    fn test_date_parsing() {
        let result = parse_date_string("2023-12-25");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());

        let result = parse_date_string("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_date_range_validation() {
        assert!(is_valid_date_range(
            Some("2023-01-01".to_string()),
            Some("2023-12-31".to_string())
        ));
        
        assert!(!is_valid_date_range(
            Some("2023-12-31".to_string()),
            Some("2023-01-01".to_string())
        ));
        
        // Prázdné hodnoty by měly být platné
        assert!(is_valid_date_range(None, None));
        assert!(is_valid_date_range(Some("2023-01-01".to_string()), None));
        assert!(is_valid_date_range(None, Some("2023-12-31".to_string())));
    }

    #[test]
    fn test_parameter_validation() {
        assert!(is_valid_limit(25));
        assert!(is_valid_limit(1));
        assert!(is_valid_limit(100));
        assert!(!is_valid_limit(0));
        assert!(!is_valid_limit(101));

        assert!(is_valid_offset(0));
        assert!(is_valid_offset(1000));
        assert!(!is_valid_offset(-1));

        assert!(is_valid_done_ratio(0));
        assert!(is_valid_done_ratio(50));
        assert!(is_valid_done_ratio(100));
        assert!(!is_valid_done_ratio(-1));
        assert!(!is_valid_done_ratio(101));

        assert!(is_valid_hours(0.1));
        assert!(is_valid_hours(8.0));
        assert!(is_valid_hours(24.0));
        assert!(!is_valid_hours(0.0));
        assert!(!is_valid_hours(24.1));
    }

    #[test]
    fn test_datetime_formatting() {
        let dt = DateTime::parse_from_rfc3339("2023-12-25T10:30:00Z").unwrap().with_timezone(&Utc);
        let formatted = format_datetime(&dt);
        assert_eq!(formatted, "25.12.2023 10:30:00 UTC");
    }

    #[test]
    fn test_date_formatting() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        let formatted = format_date(&date);
        assert_eq!(formatted, "25.12.2023");
    }
} 