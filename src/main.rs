use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use easyproject_mcp_server::{
    config::AppConfig,
    api::EasyProjectClient,
    tools::ToolRegistry,
    mcp::McpServer,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Načtení konfigurace
    let config = AppConfig::load().map_err(|e| anyhow::anyhow!("Chyba při načítání konfigurace: {}", e))?;
    
    // Validace konfigurace
    config.validate().map_err(|e| anyhow::anyhow!("Neplatná konfigurace: {}", e))?;
    
    // Inicializace logování
    init_logging(&config)?;
    
    info!("🚀 Spouštím EasyProject MCP Server v{}", config.server.version);
    info!("📡 Transport: {:?}", config.server.transport);
    info!("🌐 EasyProject URL: {}", config.easyproject.base_url);
    
    // Vytvoření API klienta
    let api_client = EasyProjectClient::new(&config).await
        .map_err(|e| anyhow::anyhow!("Chyba při vytváření API klienta: {}", e))?;
    
    // Vytvoření tool registry
    let tool_registry = ToolRegistry::new(api_client, &config);
    info!("🔧 Registrováno {} nástrojů", tool_registry.tool_count());
    
    // Vytvoření a spuštění MCP serveru
    let mut mcp_server = McpServer::new(config).await
        .map_err(|e| anyhow::anyhow!("Chyba při vytváření MCP serveru: {}", e))?;
    
    info!("✅ Server je připraven k příjmu požadavků");
    
    match mcp_server.run().await {
        Ok(_) => {
            info!("👋 Server byl ukončen");
            Ok(())
        }
        Err(e) => {
            error!("💥 Chyba serveru: {}", e);
            Err(e.into())
        }
    }
}

fn init_logging(config: &AppConfig) -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)  // Vypne ANSI escape sekvence
                .with_target(false) // Vypne target ve výpisu  
                .with_writer(std::io::stderr) // Přesměruje na stderr místo stdout
                .compact()  // Kompaktní formát
        );
    
    subscriber.init();
    
    Ok(())
} 