# EasyProject MCP Server

Model Context Protocol (MCP) server pro integraci s EasyProject API. Tento server umožňuje AI asistentům komunikovat s EasyProject platformou prostřednictvím standardizovaného rozhraní.

## 📋 Obsah

- [Funkce](#funkce)
- [Rychlé spuštění](#rychlé-spuštění)
- [Instalace](#instalace)
- [Konfigurace](#konfigurace)
- [Dostupné nástroje](#dostupné-nástroje)
- [Příklady použití](#příklady-použití)
- [Deployment](#deployment)
- [Vývoj](#vývoj)
- [Troubleshooting](#troubleshooting)

## ✨ Funkce

### Základní funkcionality
- **MCP protokol**: Plná implementace Model Context Protocol
- **Správa projektů**: Vytváření, aktualizace, mazání a seznam projektů
- **Správa úkolů**: Komplexní správa issues včetně přiřazování a označování jako dokončené
- **Správa uživatelů**: Seznam uživatelů a analýza pracovního vytížení
- **Časové sledování**: Záznam a správa časových záznamů
- **Reporting**: Generování sestav projektů a dashboard dat

### Pokročilé funkce
- **Caching**: In-memory cache s konfigurovatelným TTL
- **Rate limiting**: Ochrana před přetížením API
- **Retry logika**: Automatické opakování neúspěšných požadavků
- **Strukturované logování**: JSON formát pro monitoring
- **Konfigurovatelnost**: Rozsáhlé možnosti konfigurace

## 🚀 Rychlé spuštění

### Předpoklady

- Rust 1.70+
- Aktivní EasyProject instance
- API klíč pro EasyProject

### Základní spuštění

1. **Klonování repozitáře**:
```bash
git clone https://github.com/your-org/easyproject-mcp-server.git
cd easyproject-mcp-server
```

2. **Konfigurace**:
```bash
cp config.toml.example config.toml
# Upravte config.toml s vašimi údaji
```

3. **Nastavení environment proměnných**:
```bash
export EASYPROJECT_API_KEY="your-api-key"
export EASYPROJECT_BASE_URL="https://your-instance.easyproject.com"
```

4. **Spuštění**:
```bash
cargo run
```

## 📦 Instalace

### Pomocí Cargo

```bash
cargo install easyproject-mcp-server
```

### Pomocí Docker

```bash
docker run -d \
  --name easyproject-mcp \
  -e EASYPROJECT_API_KEY="your-key" \
  -e EASYPROJECT_BASE_URL="https://your-instance.com" \
  easyproject/mcp-server:latest
```

### Sestavení ze zdrojového kódu

```bash
git clone https://github.com/your-org/easyproject-mcp-server.git
cd easyproject-mcp-server
cargo build --release
```

## ⚙️ Konfigurace

Server používá TOML konfigurační soubor. Příklad kompletní konfigurace:

```toml
[server]
name = "EasyProject MCP Server"
version = "1.0.0"
transport = "stdio"  # stdio nebo websocket
websocket_port = 8080

[easyproject]
base_url = "https://your-instance.easyproject.com"
api_version = "v1"
auth_type = "api_key"  # api_key, oauth2, session
api_key = ""  # Doporučujeme nastavit přes ENV
api_key_header = "X-Redmine-API-Key"

[http]
timeout_seconds = 30
max_retries = 3
retry_delay_seconds = 1
user_agent = "EasyProject-MCP-Server/1.0.0"

[rate_limiting]
enabled = true
requests_per_minute = 60
burst_size = 10

[cache]
enabled = true
ttl_seconds = 300
max_entries = 1000
project_ttl = 600
user_ttl = 1800
issue_ttl = 60
time_entry_ttl = 30

[logging]
level = "info"
format = "json"
target = "stdout"

[tools.projects]
enabled = true
include_archived = false
default_limit = 25

[tools.issues]
enabled = true
default_limit = 25
include_attachments = false
include_relations = false

[tools.users]
enabled = true
default_limit = 25

[tools.time_entries]
enabled = true
default_limit = 25

[tools.reports]
enabled = true
cache_ttl = 3600
```

### Environment proměnné

| Proměnná | Popis | Povinná |
|----------|-------|---------|
| `EASYPROJECT_API_KEY` | API klíč pro EasyProject | Ano |
| `EASYPROJECT_BASE_URL` | URL EasyProject instance | Ano |
| `MCP_LOG_LEVEL` | Úroveň logování (trace, debug, info, warn, error) | Ne |

## 🛠️ Dostupné nástroje

### Správa projektů

| Nástroj | Popis |
|---------|-------|
| `list_projects` | Seznam všech projektů s filtrováním |
| `get_project` | Detail konkrétního projektu |
| `create_project` | Vytvoření nového projektu |
| `update_project` | Aktualizace existujícího projektu |
| `delete_project` | Smazání projektu |

### Správa úkolů

| Nástroj | Popis |
|---------|-------|
| `list_issues` | Seznam úkolů s filtrováním |
| `get_issue` | Detail konkrétního úkolu |
| `create_issue` | Vytvoření nového úkolu |
| `update_issue` | Aktualizace úkolu |
| `assign_issue` | Přiřazení úkolu uživateli |
| `complete_task` | Označení úkolu jako dokončený |

### Správa uživatelů

| Nástroj | Popis |
|---------|-------|
| `list_users` | Seznam všech uživatelů |
| `get_user` | Detail konkrétního uživatele |
| `get_user_workload` | Pracovní vytížení uživatele |

### Časové sledování

| Nástroj | Popis |
|---------|-------|
| `list_time_entries` | Seznam časových záznamů |
| `get_time_entry` | Detail časového záznamu |
| `log_time` | Záznam odpracovaného času |
| `update_time_entry` | Aktualizace časového záznamu |

### Reporting

| Nástroj | Popis |
|---------|-------|
| `generate_project_report` | Detailní sestava projektu |
| `get_dashboard_data` | Agregovaná data pro dashboard |

## 📖 Příklady použití

### Získání seznamu projektů

```json
{
  "method": "tools/call",
  "params": {
    "name": "list_projects",
    "arguments": {
      "limit": 10,
      "include_archived": false
    }
  }
}
```

### Vytvoření nového úkolu

```json
{
  "method": "tools/call",
  "params": {
    "name": "create_issue",
    "arguments": {
      "project_id": 1,
      "tracker_id": 1,
      "subject": "Nový úkol",
      "description": "Popis úkolu",
      "assigned_to_id": 5,
      "priority_id": 2
    }
  }
}
```

### Přiřazení úkolu

```json
{
  "method": "tools/call",
  "params": {
    "name": "assign_issue",
    "arguments": {
      "id": 123,
      "assigned_to_id": 5
    }
  }
}
```

### Označení úkolu jako dokončený

```json
{
  "method": "tools/call",
  "params": {
    "name": "complete_task",
    "arguments": {
      "id": 123,
      "done_ratio": 100
    }
  }
}
```

### Generování sestavy projektu

```json
{
  "method": "tools/call",
  "params": {
    "name": "generate_project_report",
    "arguments": {
      "project_id": 1,
      "from_date": "2023-01-01",
      "to_date": "2023-12-31",
      "include_time_entries": true,
      "include_issues": true
    }
  }
}
```

### Získání pracovního vytížení

```json
{
  "method": "tools/call",
  "params": {
    "name": "get_user_workload",
    "arguments": {
      "id": 5,
      "from_date": "2023-11-01",
      "to_date": "2023-11-30"
    }
  }
}
```

## 🚢 Deployment

### Docker

1. **Vytvoření Docker image**:
```bash
docker build -t easyproject-mcp-server .
```

2. **Spuštění kontejneru**:
```bash
docker run -d \
  --name easyproject-mcp \
  -e EASYPROJECT_API_KEY="your-key" \
  -e EASYPROJECT_BASE_URL="https://your-instance.com" \
  -p 8080:8080 \
  easyproject-mcp-server
```

### systemd (Linux)

1. **Kopírování binárky**:
```bash
sudo cp target/release/easyproject-mcp-server /usr/local/bin/
```

2. **Vytvoření systemd service**:
```bash
sudo tee /etc/systemd/system/easyproject-mcp.service > /dev/null <<EOF
[Unit]
Description=EasyProject MCP Server
After=network.target

[Service]
Type=simple
User=easyproject
WorkingDirectory=/opt/easyproject-mcp
ExecStart=/usr/local/bin/easyproject-mcp-server
Restart=always
RestartSec=10
Environment=EASYPROJECT_API_KEY=your-key
Environment=EASYPROJECT_BASE_URL=https://your-instance.com

[Install]
WantedBy=multi-user.target
EOF
```

3. **Spuštění služby**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable easyproject-mcp
sudo systemctl start easyproject-mcp
```

## 🔧 Vývoj

### Sestavení vývojové verze

```bash
cargo build
```

### Spuštění testů

```bash
# Unit testy
cargo test

# Integration testy
cargo test --test integration_tests

# Všechny testy s výstupem
cargo test -- --nocapture
```

### Spuštění s debug logováním

```bash
RUST_LOG=debug cargo run
```

### Linting a formátování

```bash
# Formátování kódu
cargo fmt

# Linting
cargo clippy -- -D warnings

# Kontrola bezpečnosti
cargo audit
```

### Generování dokumentace

```bash
cargo doc --open
```

## 🔍 Troubleshooting

### Časté problémy

#### "Connection refused" chyba

```
Chyba: Connection refused (os error 61)
```

**Řešení**: Zkontrolujte, že:
- EasyProject instance je dostupná
- URL v konfiguraci je správná
- Firewall neblokuje připojení

#### "Unauthorized" chyba

```
Chyba: 401 Unauthorized
```

**Řešení**: Zkontrolujte, že:
- API klíč je správný
- API klíč má dostatečná oprávnění
- API klíč není expirovaný

#### "Rate limit exceeded"

```
Chyba: 429 Too Many Requests
```

**Řešení**: 
- Snižte `requests_per_minute` v konfiguraci
- Zvýšte `retry_delay_seconds`
- Kontaktujte správce EasyProject instance

#### Cache problémy

Pro vymazání cache restartujte server nebo nastavte `cache.enabled = false`.

### Debug režim

Pro detailní diagnostiku spusťte server s debug logováním:

```bash
RUST_LOG=debug ./easyproject-mcp-server
```

### Logování

Server podporuje strukturované logování. Pro analýzu logů můžete použít nástroje jako `jq`:

```bash
./easyproject-mcp-server | jq '.level == "ERROR"'
```

### Health check

Server poskytuje health check endpoint (pokud je spuštěn v WebSocket módu):

```bash
curl http://localhost:8080/health
```

## 📝 Licence

MIT License. Viz [LICENSE](LICENSE) soubor pro detaily.

## 🤝 Přispívání

1. Forkněte repozitář
2. Vytvořte feature branch (`git checkout -b feature/amazing-feature`)
3. Commitněte změny (`git commit -m 'Add amazing feature'`)
4. Pushněte do branch (`git push origin feature/amazing-feature`)
5. Otevřete Pull Request

## 📞 Podpora

- **Issues**: [GitHub Issues](https://github.com/your-org/easyproject-mcp-server/issues)
- **Diskuze**: [GitHub Discussions](https://github.com/your-org/easyproject-mcp-server/discussions)
- **Email**: support@your-org.com

## 🗺️ Roadmap

- [ ] WebSocket real-time notifikace
- [ ] Plugin systém
- [ ] Batch operace
- [ ] Prometheus metrics
- [ ] GraphQL endpoint
- [ ] Webhooks podpora

---

**Vytvořeno s ❤️ pro EasyProject komunitu**