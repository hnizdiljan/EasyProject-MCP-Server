# 🚀 EasyProject MCP Server - Deployment Guide

## ⚠️ Build Issues?

**Pokud máte problémy s ring crate nebo buildem, přečtěte si [BUILD-GUIDE.md](BUILD-GUIDE.md)** - obsahuje řešení všech build problémů včetně GitHub Actions!

## Deployment skripty

Máte k dispozici dva PowerShell skripty pro deployment:

### 1. `deploy.ps1` - Inteligentní deployment skript

**Použití:**
```powershell
# Použije existující EXE (doporučeno)
.\deploy.ps1 -SkipBuild

# Vynutí nový build
.\deploy.ps1 -Force

# Standardní režim (build jen pokud EXE neexistuje)
.\deploy.ps1
```

**Funkce:**
- ✅ Inteligentní build strategie (MSVC toolchain → standardní build → nízké optimalizace)
- ✅ Vytvoří kompletní deployment balíček
- ✅ Aktualizuje README.md s aktuální velikostí a datem
- ✅ Vytvoří MCP config template
- ✅ Fallback na existující EXE při selhání buildu

### 2. `quick-deploy.ps1` - Rychlý deployment

**Použití:**
```powershell
.\quick-deploy.ps1
```

**Funkce:**
- ⚡ Rychlé zkopírování existujícího EXE
- 📦 Minimální výstup
- 🎯 Pouze pro rychlé deployment

### 3. `setup-build-tools.ps1` - Setup build prostředí

**Použití:**
```powershell
.\setup-build-tools.ps1
```

**Funkce:**
- 🔧 Instaluje MSVC toolchain pro Rust
- 📥 Automaticky nainstaluje Visual Studio Build Tools
- ✅ Testuje build prostředí

## 📁 Výsledek

Po spuštění kteréhokoliv skriptu se vytvoří složka `deployment/` obsahující:

```
deployment/
├── easyproject-mcp-server.exe    (15.29 MB - single-file EXE)
├── README.md                     (deployment návod)
└── cursor-mcp-config.json        (MCP config template)
```

## 🔧 Single-File Deployment

EXE soubor je **kompletně samostatný** a obsahuje:
- ✅ Všechny Rust dependencies
- ✅ TLS support (rust-native, bez OpenSSL)
- ✅ JSON parsing
- ✅ HTTP client
- ✅ Async runtime

**Systémové požadavky:**
- Windows 10/11 x64
- Visual C++ Redistributable 2019+ (obvykle již nainstalován)

## 📝 Konfigurace pro Cursor

1. **Zkopírujte EXE** na cílový systém (např. `C:\Program Files\EasyProject\`)

2. **Upravte Cursor MCP config:**

```json
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
```

3. **Restart Cursor**

## 🎯 Použití v praxi

### Vývojový workflow:
```powershell
# Po změnách v kódu
cargo build --release
.\quick-deploy.ps1

# Nebo pro kompletní deployment s dokumentací
.\deploy.ps1 -SkipBuild
```

### Produkční deployment:
```powershell
# Vytvoří optimalizovaný build + kompletní balíček
.\deploy.ps1 -Force
```

## 🚀 Testování deployment

Po konfiguraci v Cursor můžete použít:

```