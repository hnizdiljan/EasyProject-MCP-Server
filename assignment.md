# Zadání pro vytvoření MCP serveru pro EasyProject

## Cíl projektu

Vytvořit Model Context Protocol (MCP) server, který umožní AI asistentům komunikovat s EasyProject API prostřednictvím standardizovaného rozhraní. Server bude poskytovat nástroje pro správu projektů, úkolů, uživatelů a dalších funkcionalit dostupných v EasyProject platformě.

## Technické požadavky

### Doporučený programovací jazyk

**Rust** - z následujících důvodů:

- **Výkon**: Kompilovaný jazyk s minimální runtime overhead
- **Bezpečnost paměti**: Eliminuje běžné chyby jako buffer overflow
- **Asynchronní zpracování**: Výborná podpora pro async/await s tokio
- **Ekosystém**: Kvalitní knihovny pro HTTP klienty (reqwest), JSON (serde), WebSocket komunikaci
- **Nízká spotřeba zdrojů**: Ideální pro dlouhodobě běžící servery

### Architektura MCP serveru

#### 1. Komunikační vrstva

- Implementace MCP protokolu přes stdio/WebSocket
- Podpora pro standardní MCP zprávy (initialize, tools/list, tools/call)
- Error handling a logování

#### 2. API klient

- HTTP klient pro komunikaci s EasyProject API
- Autentifikace (API klíče, OAuth2, session management)
- Rate limiting a retry mechanismy
- Response caching pro optimalizaci

#### 3. Nástroje (Tools)

Na základě EasyProject Swagger dokumentace implementovat nástroje pro:

**Správa projektů**:

- `list_projects` - seznam všech projektů
- `get_project` - detail konkrétního projektu
- `create_project` - vytvoření nového projektu
- `update_project` - aktualizace projektu
- `delete_project` - smazání projektu

**Správa úkolů**:

- `list_tasks` - seznam úkolů s filtrováním
- `get_task` - detail úkolu
- `create_task` - vytvoření úkolu
- `update_task` - aktualizace úkolu
- `assign_task` - přiřazení úkolu
- `complete_task` - označení úkolu jako dokončený

**Správa uživatelů**:

- `list_users` - seznam uživatelů
- `get_user` - profil uživatele
- `get_user_workload` - pracovní vytížení uživatele

**Časové sledování**:

- `log_time` - záznam času
- `get_time_entries` - seznam časových záznamů
- `update_time_entry` - aktualizace záznamu

**Reporting**:

- `generate_project_report` - sestavy k projektům
- `get_dashboard_data` - data pro dashboard

#### 4. Konfigurace

Konfigurační soubor (TOML/YAML) pro:

- EasyProject API endpoint
- Autentifikační údaje
- Cache nastavení
- Logging level
- Rate limiting parametry

## Dostupné zdroje

K dispozici máme soubor **`easy_swagger.yml`** obsahující kompletní definici EasyProject API včetně:

- Všech dostupných endpointů a jejich parametrů
- Datových modelů a schémat
- Způsobů autentifikace
- Response formátů a error kódů
- API verzí a kompatibility

## Implementační kroky

### 1. Analýza Swagger dokumentace

- Prostudovat soubor `easy_swagger.yml`
- Identifikovat klíčové endpointy a jejich parametry z definice
- Zmapovat datové modely a vztahy podle schémat
- Implementovat autentifikační mechanismus podle specifikace

### 2. Základní struktura

```
easyproject-mcp/
├── src/
│   ├── main.rs
│   ├── mcp/          # MCP protokol implementace
│   ├── api/          # EasyProject API klient
│   ├── tools/        # MCP nástroje
│   ├── config/       # Konfigurace
│   └── utils/        # Pomocné funkce
├── easy_swagger.yml  # EasyProject API definice
├── Cargo.toml
├── config.toml
└── README.md
```

### 3. MCP protokol

- Implementace MCP server traits
- Message handling (JSON-RPC 2.0)
- Tools registration a execution
- Error handling podle MCP specifikace

### 4. API integrace

- HTTP klient s retry logikou
- Request/response modely generované ze `easy_swagger.yml`
- Autentifikace podle specifikace ve Swagger souboru
- Error mapping z API na MCP errors

### 5. Testování

- Unit testy pro jednotlivé komponenty
- Integration testy s mock EasyProject API
- End-to-end testy s reálným API
- Performance testy pro high-load scénáře

## Pokročilé funkce

### 1. Caching

- In-memory cache pro často používaná data
- TTL nastavení podle typu dat
- Cache invalidation při změnách

### 2. Batch operace

- Seskupování více API volání
- Bulk operace tam, kde je podporuje API
- Optimalizace pro velké datové sady

### 3. Real-time aktualizace

- WebSocket připojení pro live updates
- Notifikace o změnách v projektech/úkolech
- Event streaming pro monitoring

### 4. Extensibilita

- Plugin systém pro custom nástroje
- Webhook support pro externí integrace
- Konfigurovatelné transformace dat

## Dokumentace a deployment

### 1. Dokumentace

- API dokumentace pro všechny MCP nástroje
- Příklady použití pro různé use cases
- Configuration guide
- Troubleshooting guide

### 2. Deployment

- Docker container pro snadné nasazení
- systemd service files pro Linux
- Environment variable konfigurace
- Health check endpoints

### 3. Monitoring

- Structured logging (JSON)
- Metrics export (Prometheus)
- Performance monitoring
- Error tracking


## Závěr

Tento MCP server poskytne robustní a efektivní most mezi AI asistenty a EasyProject platformou, umožní automatizaci projektového managementu a zvýší produktivitu týmů.