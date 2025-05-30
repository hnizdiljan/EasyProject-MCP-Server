# Jednoduchý build script pro EasyProject MCP Server
Write-Host "Building EasyProject MCP Server - Optimized Release" -ForegroundColor Green

# Vyčištění předchozích buildů
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
cargo clean

# Build s optimalizacemi
Write-Host "Building with optimizations..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-C target-cpu=native -C link-arg=-s"
cargo build --release

if ($LASTEXITCODE -eq 0) {
    $exePath = "target\release\easyproject-mcp-server.exe"
    
    if (Test-Path $exePath) {
        $fileSize = (Get-Item $exePath).Length
        $fileSizeMB = [math]::Round($fileSize / 1MB, 2)
        
        Write-Host "Build successful!" -ForegroundColor Green
        Write-Host "EXE Size: $fileSizeMB MB" -ForegroundColor Cyan
        Write-Host "Location: $exePath" -ForegroundColor Cyan
        
        # Zkopírování do deployment složky
        $deployDir = "deployment"
        if (!(Test-Path $deployDir)) {
            New-Item -ItemType Directory -Path $deployDir
        }
        
        Copy-Item $exePath "$deployDir\easyproject-mcp-server.exe" -Force
        
        # Vytvoření jednoduchého README
        $readmeText = "# EasyProject MCP Server - Deployment`n`n"
        $readmeText += "Single-File Deployment ready!`n`n"
        $readmeText += "Requirements:`n"
        $readmeText += "* Windows 10/11 x64`n"
        $readmeText += "* Visual C++ Redistributable 2019+`n`n"
        $readmeText += "Usage:`n"
        $readmeText += "1. Copy easyproject-mcp-server.exe to target system`n"
        $readmeText += "2. Set environment variables`n"
        $readmeText += "3. Run easyproject-mcp-server.exe`n`n"
        $readmeText += "Build info:`n"
        $readmeText += "* Size: $fileSizeMB MB`n"
        $readmeText += "* Built: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n"
        $readmeText += "* Optimizations: Size optimized, LTO enabled`n"
        
        $readmeText | Out-File -FilePath "$deployDir\README.md" -Encoding UTF8
        
        Write-Host "Deployment package created in: $deployDir\" -ForegroundColor Green
        Write-Host "See README.md for instructions" -ForegroundColor Green
        
    } else {
        Write-Host "EXE file not found!" -ForegroundColor Red
    }
} else {
    Write-Host "Build failed!" -ForegroundColor Red
} 