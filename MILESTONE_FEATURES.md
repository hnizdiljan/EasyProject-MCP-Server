# Milníky (Versions) - Nové funkcionality MCP serveru

## Přehled

MCP server byl rozšířen o podporu pro práci s milníky (versions) v EasyProject systému. Milníky reprezentují verze nebo časové body v rámci projektů.

## Nové nástroje (Tools)

### 1. `list_milestones`
Získá seznam všech milníků v systému s možností filtrování.

**Parametry:**
- `limit` (volitelný): Maximální počet milníků k vrácení (výchozí: 25, maximum: 100)
- `offset` (volitelný): Počet milníků k přeskočení pro stránkování
- `project_id` (volitelný): ID projektu pro filtrování milníků
- `status` (volitelný): Status milníku (`"open"`, `"locked"`, `"closed"`)
- `easy_query_q` (volitelný): Volný text pro vyhledávání

**Příklad použití:**
```json
{
  "limit": 50,
  "project_id": 123,
  "status": "open"
}
```

### 2. `get_milestone`
Získá detail konkrétního milníku podle ID.

**Parametry:**
- `id` (povinný): ID milníku

**Příklad použití:**
```json
{
  "id": 456
}
```

### 3. `create_milestone`
Vytvoří nový milník v zadaném projektu.

**Parametry:**
- `project_id` (povinný): ID projektu, kde se má milník vytvořit
- `name` (povinný): Název milníku
- `description` (volitelný): Popis milníku
- `effective_date` (volitelný): Datum začátku milníku (YYYY-MM-DD)
- `due_date` (volitelný): Datum ukončení milníku (YYYY-MM-DD)
- `status` (volitelný): Status milníku (`"open"`, `"locked"`, `"closed"`)
- `sharing` (volitelný): Nastavení sdílení (`"none"`, `"descendants"`, `"hierarchy"`, `"tree"`, `"system"`)
- `default_project_version` (volitelný): Zda je toto výchozí verze projektu
- `easy_external_id` (volitelný): Externí ID pro integraci

**Příklad použití:**
```json
{
  "project_id": 123,
  "name": "Verze 2.0",
  "description": "Hlavní release s novými funkcemi",
  "due_date": "2024-06-30",
  "status": "open"
}
```

### 4. `update_milestone`
Aktualizuje existující milník.

**Parametry:**
- `id` (povinný): ID milníku k aktualizaci
- Všechny ostatní parametry jsou volitelné a odpovídají parametrům z `create_milestone`

**Příklad použití:**
```json
{
  "id": 456,
  "status": "closed",
  "due_date": "2024-07-15"
}
```

### 5. `delete_milestone`
Smaže existující milník.

**Parametry:**
- `id` (povinný): ID milníku k smazání

**Příklad použití:**
```json
{
  "id": 456
}
```

## Konfigurace

V konfiguračním souboru byla přidána nová sekce pro milníky:

```toml
[tools.milestones]
enabled = true
default_limit = 25
```

### Konfigurační parametry:
- `enabled`: Zapne/vypne nástroje pro milníky
- `default_limit`: Výchozí limit pro stránkování

## API Endpointy

Nástroje využívají následující EasyProject API endpointy:

- `GET /versions.json` - seznam milníků
- `GET /versions/{id}.json` - detail milníku
- `POST /projects/{project_id}/versions.json` - vytvoření milníku
- `PUT /versions/{id}.json` - aktualizace milníku
- `DELETE /versions/{id}.json` - smazání milníku

## Datové struktury

### Version (Milestone) Model
```rust
pub struct Version {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub wiki_page_title: Option<String>,
    pub sharing: Option<String>,
    pub default_project_version: Option<bool>,
    pub easy_external_id: Option<String>,
    pub project: Option<ProjectReference>,
    pub easy_version_category: Option<Value>,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
}
```

## Cache a optimalizace

- Všechny GET operace využívají cache pro rychlejší odezvu
- Cache se automaticky invaliduje při operacích CUD (Create, Update, Delete)
- Implementováno rate limiting pro prevenci přetížení API

## Bezpečnost

- Všechny požadavky využívají autentifikaci přes API klíč
- Validace vstupních parametrů
- Ošetření chybových stavů

## Integrace s MCP protokolem

Nástroje jsou plně kompatibilní s MCP (Model Context Protocol) a mohou být využívány AI asistenty pro:

- Správu životního cyklu projektů
- Plánování releaseů
- Sledování milníků
- Reporting a analýzu postupu projektu

## Chybové stavy

Nástroje správně ošetřují následující chybové stavy:

- Neplatné ID milníku/projektu (404 Not Found)
- Nedostatečná oprávnění (403 Forbidden)
- Neplatné vstupní parametry (422 Unprocessable Entity)
- Síťové chyby a timeouty
- Rate limiting (429 Too Many Requests)

## Logování

Všechny operace jsou logovány na úrovni INFO/DEBUG pro snadné ladění a monitoring. 