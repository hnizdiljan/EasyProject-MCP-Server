use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub easyproject: EasyProjectConfig,
    pub http: HttpConfig,
    pub rate_limiting: RateLimitingConfig,
    pub cache: CacheConfig,
    pub logging: LoggingConfig,
    pub tools: ToolsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub transport: TransportType,
    pub websocket_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportType {
    Stdio,
    Websocket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EasyProjectConfig {
    pub base_url: String,
    pub api_version: String,
    pub auth_type: AuthType,
    pub api_key: Option<String>,
    pub api_key_header: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    ApiKey,
    OAuth2,
    Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_entries: u64,
    pub project_ttl: u64,
    pub user_ttl: u64,
    pub issue_ttl: u64,
    pub time_entry_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub projects: ProjectToolConfig,
    pub issues: IssueToolConfig,
    pub users: UserToolConfig,
    pub time_entries: TimeEntryToolConfig,
    pub reports: ReportToolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectToolConfig {
    pub enabled: bool,
    pub include_archived: bool,
    pub default_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueToolConfig {
    pub enabled: bool,
    pub default_limit: u32,
    pub include_attachments: bool,
    pub include_relations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToolConfig {
    pub enabled: bool,
    pub default_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryToolConfig {
    pub enabled: bool,
    pub default_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportToolConfig {
    pub enabled: bool,
    pub cache_ttl: u64,
}

impl AppConfig {
    /// Načte konfiguraci ze souboru a environment proměnných
    pub fn load() -> Result<Self> {
        // Zkusí načíst konfiguraci normálně
        let result = Self::try_load_config();
        
        match result {
            Ok(config) => Ok(config),
            Err(_) => {
                // Při chybě použije výchozí konfiguraci
                tracing::warn!("Použita výchozí konfigurace kvůli chybě deserializace");
                let mut config = Self::default();
                
                // Nastaví API klíč z environment proměnné
                if let Ok(api_key) = std::env::var("EASYPROJECT_API_KEY") {
                    config.easyproject.api_key = Some(api_key);
                }
                
                // Nastaví base URL z environment proměnné
                if let Ok(base_url) = std::env::var("EASYPROJECT_BASE_URL") {
                    config.easyproject.base_url = base_url;
                }
                
                Ok(config)
            }
        }
    }
    
    fn try_load_config() -> Result<Self> {
        let mut settings = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("EASYPROJECT_MCP"))
            .build()
            .context("Nepodařilo se načíst konfiguraci")?;

        // Přepsat API klíč z environment proměnné pokud existuje
        if let Ok(api_key) = std::env::var("EASYPROJECT_API_KEY") {
            settings.set("easyproject.api_key", api_key)
                .context("Nepodařilo se nastavit API klíč z environment proměnné")?;
        }

        let config: AppConfig = settings
            .try_deserialize()
            .context("Nepodařilo se deserializovat konfiguraci")?;

        Ok(config)
    }

    /// Validuje konfiguraci
    pub fn validate(&self) -> Result<()> {
        // Validace URL
        Url::parse(&self.easyproject.base_url)
            .context("Neplatná base_url pro EasyProject")?;

        // Validace autentifikace
        match self.easyproject.auth_type {
            AuthType::ApiKey => {
                if self.easyproject.api_key.is_none() || self.easyproject.api_key.as_ref().unwrap().is_empty() {
                    anyhow::bail!("API klíč je povinný pro auth_type = 'api_key'");
                }
            }
            AuthType::OAuth2 => {
                if self.easyproject.client_id.is_none() || self.easyproject.client_secret.is_none() {
                    anyhow::bail!("client_id a client_secret jsou povinné pro OAuth2");
                }
            }
            AuthType::Session => {
                // Session auth zatím není implementován
                anyhow::bail!("Session autentifikace zatím není podporována");
            }
        }

        // Validace WebSocket portu
        if matches!(self.server.transport, TransportType::Websocket) {
            if self.server.websocket_port.is_none() {
                anyhow::bail!("websocket_port je povinný pro WebSocket transport");
            }
        }

        // Validace HTTP nastavení
        if self.http.timeout_seconds == 0 {
            anyhow::bail!("timeout_seconds musí být větší než 0");
        }

        if self.http.max_retries > 10 {
            anyhow::bail!("max_retries by neměl být větší než 10");
        }

        Ok(())
    }

    /// Vrátí timeout pro HTTP požadavky
    pub fn http_timeout(&self) -> Duration {
        Duration::from_secs(self.http.timeout_seconds)
    }

    /// Vrátí delay pro retry HTTP požadavků
    pub fn retry_delay(&self) -> Duration {
        Duration::from_secs(self.http.retry_delay_seconds)
    }

    /// Vrátí TTL pro cache podle typu entity
    pub fn cache_ttl_for_entity(&self, entity_type: &str) -> Duration {
        let seconds = match entity_type {
            "project" => self.cache.project_ttl,
            "user" => self.cache.user_ttl,
            "issue" => self.cache.issue_ttl,
            "time_entry" => self.cache.time_entry_ttl,
            _ => self.cache.ttl_seconds,
        };
        Duration::from_secs(seconds)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                name: "EasyProject MCP Server".to_string(),
                version: "1.0.0".to_string(),
                transport: TransportType::Stdio,
                websocket_port: Some(8080),
            },
            easyproject: EasyProjectConfig {
                base_url: "https://your-easyproject-instance.com".to_string(),
                api_version: "v1".to_string(),
                auth_type: AuthType::ApiKey,
                api_key: None,
                api_key_header: "X-Redmine-API-Key".to_string(),
                client_id: None,
                client_secret: None,
                redirect_uri: None,
                scopes: vec![],
            },
            http: HttpConfig {
                timeout_seconds: 30,
                max_retries: 3,
                retry_delay_seconds: 1,
                user_agent: "EasyProject-MCP-Server/1.0.0".to_string(),
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                requests_per_minute: 60,
                burst_size: 10,
            },
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 300,
                max_entries: 1000,
                project_ttl: 600,
                user_ttl: 1800,
                issue_ttl: 60,
                time_entry_ttl: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Json,
                target: "stdout".to_string(),
            },
            tools: ToolsConfig {
                projects: ProjectToolConfig {
                    enabled: true,
                    include_archived: false,
                    default_limit: 25,
                },
                issues: IssueToolConfig {
                    enabled: true,
                    default_limit: 25,
                    include_attachments: false,
                    include_relations: false,
                },
                users: UserToolConfig {
                    enabled: true,
                    default_limit: 25,
                },
                time_entries: TimeEntryToolConfig {
                    enabled: true,
                    default_limit: 25,
                },
                reports: ReportToolConfig {
                    enabled: true,
                    cache_ttl: 3600,
                },
            },
        }
    }
} 