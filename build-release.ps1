# Build script pro optimalizovaný EasyProject MCP Server release
# Použití: .\build-release.ps1

Write-Host "🚀 Building EasyProject MCP Server - Optimized Release" -ForegroundColor Green

# Vyčištění předchozích buildů
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
cargo clean

# Build s optimalizacemi
Write-Host "🔧 Building with optimizations..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-C target-cpu=native -C link-arg=-s"
cargo build --release

if ($LASTEXITCODE -eq 0) {
    $exePath = "target\release\easyproject-mcp-server.exe"
    
    if (Test-Path $exePath) {
        $fileSize = (Get-Item $exePath).Length
        $fileSizeMB = [math]::Round($fileSize / 1MB, 2)
        
        Write-Host "✅ Build successful!" -ForegroundColor Green
        Write-Host "📦 EXE Size: $fileSizeMB MB" -ForegroundColor Cyan
        Write-Host "📍 Location: $exePath" -ForegroundColor Cyan
        
        # Zkopírování do deployment složky
        $deployDir = "deployment"
        if (!(Test-Path $deployDir)) {
            New-Item -ItemType Directory -Path $deployDir
        }
        
        Copy-Item $exePath "$deployDir\easyproject-mcp-server.exe" -Force
        
        # Vytvoření deployment readme
        $readmeContent = @"
# EasyProject MCP Server - Deployment

## Single-File Deployment

Tento EXE soubor je samostatny a obsahuje vsechny zavislosti.

### Systemove pozadavky:
- Windows 10/11 x64
- Visual C++ Redistributable 2019+

### Pouziti:
1. Zkopirujte easyproject-mcp-server.exe na cilovy system
2. Nastavte environment variables
3. Spustte easyproject-mcp-server.exe

### Build info:
- Size: $fileSizeMB MB
- Built: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
- Optimizations: Size optimized, LTO enabled, symbols stripped
- TLS: RustTLS (no OpenSSL dependencies)
"@
        
        $readmeContent | Out-File -FilePath "$deployDir\README.md" -Encoding UTF8
        
        # Vytvoření MCP config template
        $mcpConfigContent = @"
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
"@
        
        $mcpConfigContent | Out-File -FilePath "$deployDir\cursor-mcp-config.json" -Encoding UTF8
        
        Write-Host "📁 Deployment package created in: $deployDir\" -ForegroundColor Green
        Write-Host "📄 See README.md for deployment instructions" -ForegroundColor Green
        Write-Host "📄 See cursor-mcp-config.json for MCP configuration template" -ForegroundColor Green
        
        # Testování závislostí
        Write-Host "🔍 Testing dependencies..." -ForegroundColor Yellow
        try {
            $dependencies = & dumpbin /dependents $exePath 2>$null
            if ($dependencies -and ($dependencies | Select-String "\.dll")) {
                Write-Host "⚠️  External dependencies found:" -ForegroundColor Yellow
                $dependencies | Select-String "\.dll" | ForEach-Object { Write-Host "   - $($_.Line.Trim())" -ForegroundColor Yellow }
            } else {
                Write-Host "✅ No external DLL dependencies detected" -ForegroundColor Green
            }
        } catch {
            Write-Host "ℹ️  Could not check dependencies (dumpbin not available)" -ForegroundColor Cyan
        }
        
    } else {
        Write-Host "❌ EXE file not found!" -ForegroundColor Red
    }
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
} 