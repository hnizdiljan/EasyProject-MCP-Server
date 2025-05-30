# Nastavení EasyProject MCP Server v Cursoru

## ✅ Úspěšně zkompilováno!

Server byl úspěšně zkompilován s **GNU toolchain** (MinGW-w64), což vyřešilo problémy s Visual Studio Build Tools.

**Executable:** `target\release\easyproject-mcp-server.exe` (15.9 MB)

## 🔧 Požadavky

1. **Rust s GNU toolchain** (již nainstalováno):
   ```bash
   rustup default stable-x86_64-pc-windows-gnu
   ```

2. **EasyProject API klíč** a URL instance

## ⚙️ Konfigurace Cursor

### 1. Otevřete nastavení Cursor

- **Windows/Linux**: `Ctrl + ,`
- **macOS**: `Cmd + ,`

### 2. Najděte sekci "MCP Servers"

Nebo použijte search a hledejte "mcp"

### 3. Přidejte konfiguraci serveru

```json
{
  "mcpServers": {
    "easyproject": {
      "command": "C:\\Users\\hnizd\\source\\repos\\hnizdiljan\\EasyProject-MCP-Server\\target\\release\\easyproject-mcp-server.exe",
      "args": [],
      "env": {
        "EASYPROJECT_API_KEY": "váš-api-klíč",
        "EASYPROJECT_BASE_URL": "https://vaše-instance.easyproject.com",
        "MCP_LOG_LEVEL": "info"
      }
    }
  }
}
```

**⚠️ Důležité:** Upravte cestu k executable podle vaší instalace!

### 4. Alternativní konfigurace s config.toml

Můžete také vytvořit `config.toml` soubor ve složce projektu:

```toml
[server]
name = "EasyProject MCP Server"
version = "0.1.0"
transport = "stdio"

[easyproject]
base_url = "https://vaše-instance.easyproject.com"
api_key = "váš-api-klíč"

[cache]
enabled = true
ttl_seconds = 300
max_entries = 1000

[rate_limiting]
enabled = true
requests_per_minute = 60
burst_size = 10

[http]
timeout_seconds = 30
user_agent = "EasyProject-MCP-Server/0.1.0"
```

A pak použít jednodušší konfiguraci v Cursor:

```json
{
  "mcpServers": {
    "easyproject": {
      "command": "C:\\cesta\\k\\easyproject-mcp-server.exe",
      "args": []
    }
  }
}
```

## 🚀 Spuštění a testování

### 1. Restartujte Cursor

Po přidání konfigurace restartujte Cursor.

### 2. Ověřte připojení

V Cursor chat zkuste:

```
Zobraz mi všechny aktivní projekty v EasyProject
```

### 3. Dostupné nástroje

Server poskytuje tyto nástroje:

**Projekty:**
- `list_projects` - seznam projektů
- `get_project` - detail projektu
- `create_project` - vytvoření projektu
- `update_project` - aktualizace projektu
- `delete_project` - smazání projektu

**Úkoly:**
- `list_issues` - seznam úkolů
- `get_issue` - detail úkolu
- `create_issue` - vytvoření úkolu
- `update_issue` - aktualizace úkolu
- `assign_issue` - přiřazení úkolu
- `complete_task` - dokončení úkolu

**Uživatelé:**
- `list_users` - seznam uživatelů
- `get_user` - detail uživatele
- `get_user_workload` - pracovní vytížení

**Časové záznamy:**
- `list_time_entries` - seznam časových záznamů
- `log_time` - záznam času

**Reporting:**
- `generate_project_report` - sestavy projektů
- `get_dashboard_data` - dashboard data

## 🐛 Troubleshooting

### Server se nespustí

1. **Zkontrolujte cestu k executable**
2. **Ověřte API klíč a URL**
3. **Zkontrolujte logy Cursor** (Developer Tools → Console)

### Chyby kompilace (pro budoucí úpravy)

Pokud budete upravovat kód:

```bash
# Vyčistěte build
cargo clean

# Zkompilujte znovu
cargo build --release
```

### GNU vs MSVC toolchain

Projekt je nakonfigurován pro **GNU toolchain**, což eliminuje potřebu Visual Studio Build Tools:

```bash
# Ověření aktuálního toolchain
rustup show

# Přepnutí na GNU (pokud potřeba)
rustup default stable-x86_64-pc-windows-gnu
```

## 📝 Poznámky

- Server běží v **stdio** módu pro komunikaci s Cursor
- **Cache** je aktivní pro optimalizaci výkonu
- **Rate limiting** chrání před přetížením API
- Všechny nástroje mají **validaci vstupů** a **error handling**

## 🎯 Příklady použití

Viz `cursor-usage-examples.md` pro konkrétní příklady práce s MCP serverem v Cursor.

## 🔐 Bezpečnost

- **Nikdy necommitujte API klíče** do veřejných repozitářů
- **Používejte environment proměnné** pro citlivé údaje
- **Pravidelně rotujte** API klíče
- **Omezte oprávnění** API klíčů na minimum potřebné

## 📚 Další zdroje

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [Cursor MCP Documentation](https://docs.cursor.com/mcp)
- [EasyProject API Documentation](https://docs.easyproject.com/) 