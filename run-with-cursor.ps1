# EasyProject MCP Server - Script pro spuštění s Cursor
# Použití: .\run-with-cursor.ps1

param(
    [string]$ApiKey = $env:EASYPROJECT_API_KEY,
    [string]$BaseUrl = $env:EASYPROJECT_BASE_URL,
    [string]$LogLevel = "info"
)

Write-Host "🚀 EasyProject MCP Server Setup" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green

# Kontrola Rust instalace
Write-Host "📋 Kontroluji Rust instalaci..." -ForegroundColor Yellow
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust není nainstalován!" -ForegroundColor Red
    Write-Host "   Nainstalujte Rust z https://rustup.rs/" -ForegroundColor Red
    Write-Host "   Nebo použijte: winget install Rustlang.Rustup" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Rust je nainstalován" -ForegroundColor Green

# Kontrola API klíče
if (-not $ApiKey) {
    Write-Host "❌ Chybí EASYPROJECT_API_KEY!" -ForegroundColor Red
    Write-Host "   Nastavte environment proměnnou nebo předejte jako parametr" -ForegroundColor Red
    Write-Host "   Příklad: .\run-with-cursor.ps1 -ApiKey 'your-key' -BaseUrl 'https://your-instance.com'" -ForegroundColor Red
    exit 1
}

if (-not $BaseUrl) {
    Write-Host "❌ Chybí EASYPROJECT_BASE_URL!" -ForegroundColor Red
    Write-Host "   Nastavte environment proměnnou nebo předejte jako parametr" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Konfigurace je v pořádku" -ForegroundColor Green

# Kompilace
Write-Host "🔨 Kompiluji MCP server..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Kompilace selhala!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Kompilace dokončena" -ForegroundColor Green

# Zobrazení konfigurace pro Cursor
Write-Host ""
Write-Host "📋 Konfigurace pro Cursor:" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan

$currentPath = (Get-Location).Path
$executablePath = "$currentPath\target\release\easyproject-mcp-server.exe"

$cursorConfig = @{
    mcpServers = @{
        easyproject = @{
            command = $executablePath
            args = @()
            env = @{
                EASYPROJECT_API_KEY = $ApiKey
                EASYPROJECT_BASE_URL = $BaseUrl
                MCP_LOG_LEVEL = $LogLevel
            }
        }
    }
} | ConvertTo-Json -Depth 10

Write-Host $cursorConfig -ForegroundColor White

# Uložení konfigurace do souboru
$configFile = "cursor-mcp-config.json"
$cursorConfig | Out-File -FilePath $configFile -Encoding UTF8
Write-Host ""
Write-Host "✅ Konfigurace uložena do $configFile" -ForegroundColor Green

# Instrukce pro Cursor
Write-Host ""
Write-Host "🎯 Další kroky:" -ForegroundColor Cyan
Write-Host "==============" -ForegroundColor Cyan
Write-Host "1. Otevřete Cursor nastavení (Ctrl + ,)" -ForegroundColor White
Write-Host "2. Najděte sekci 'MCP Servers'" -ForegroundColor White
Write-Host "3. Zkopírujte výše uvedenou konfiguraci" -ForegroundColor White
Write-Host "4. Nebo použijte soubor: $configFile" -ForegroundColor White
Write-Host "5. Restartujte Cursor" -ForegroundColor White

# Test spuštění
Write-Host ""
Write-Host "🧪 Testování serveru..." -ForegroundColor Yellow

$env:EASYPROJECT_API_KEY = $ApiKey
$env:EASYPROJECT_BASE_URL = $BaseUrl
$env:MCP_LOG_LEVEL = $LogLevel

Write-Host "Spouštím server v test módu..." -ForegroundColor Yellow
Write-Host "Pro ukončení použijte Ctrl+C" -ForegroundColor Yellow
Write-Host ""

& $executablePath 