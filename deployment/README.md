# EasyProject MCP Server - Single-File Deployment

## Ready for Deployment!

Tento EXE soubor (3.21 MB) je samostatny a obsahuje vsechny zavislosti.

## Systemove pozadavky
- Windows 10/11 x64
- Visual C++ Redistributable 2019+

## Pouziti

### 1. Zkopirujte EXE soubor
easyproject-mcp-server.exe
na cilovy system do libovolne slozky.

### 2. Konfigurace pro Cursor MCP

Upravte konfiguraci v Cursor (cursor-mcp-config.json):

{
  "mcpServers": {
    "easyproject": {
      "command": "C:\\path\\to\\easyproject-mcp-server.exe",
      "args": [],
      "env": {
        "EASYPROJECT_API_KEY": "your-api-key-here",
        "EASYPROJECT_BASE_URL": "https://your-instance.easyproject.com"
      }
    }
  }
}

### 3. Nastavte environment variables

Nahradte hodnoty:
- EASYPROJECT_API_KEY - vas API klic z EasyProject
- EASYPROJECT_BASE_URL - URL vasi EasyProject instance

### 4. Restart Cursor

Po konfiguraci restartujte Cursor.

## Testovani

Po spusteni muzete v Cursor pouzit nastroje jako:
- list_projects - seznam projektu
- list_issues - seznam ukolu
- create_issue - vytvoreni noveho ukolu
- log_time - logovani casu

## Build informace
- Velikost: 3.21 MB
- Datum build: 2025-05-31 01:08:56
- Single-file deployment
- TLS: Rust-native

## Troubleshooting

Pokud se server nespusti:
1. Zkontrolujte Visual C++ Redistributable 2019+
2. Overte spravnost API klice a URL
3. Zkontrolujte opravneni EXE souboru

Pro ladeni spustte EXE z Command Line.
