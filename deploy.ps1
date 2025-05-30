# EasyProject MCP Server - Smart Deployment Script
# Pouziti: .\deploy.ps1

param(
    [switch]$Force,     # Vynuti novy build
    [switch]$SkipBuild  # Pouzije existujici EXE
)

Write-Host "EasyProject MCP Server - Smart Deployment" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green

$deployDir = "deployment"
$targetExe = "target\release\easyproject-mcp-server.exe"
$deployExe = "$deployDir\easyproject-mcp-server.exe"

function Get-FileSizeMB($filePath) {
    if (Test-Path $filePath) {
        $size = (Get-Item $filePath).Length
        return [math]::Round($size / 1MB, 2)
    }
    return 0
}

function Clear-RingCrateCache {
    Write-Host "  Clearing ring crate cache..." -ForegroundColor Yellow
    try {
        $ringPath = "$env:CARGO_HOME\registry\src\index.crates.io-1949cf8c6b5b557f\ring-0.17.14"
        if (Test-Path $ringPath) {
            Remove-Item -Recurse -Force $ringPath -ErrorAction SilentlyContinue
            Write-Host "  Ring cache cleared" -ForegroundColor Cyan
        }
        
        # Vymazat temp build soubory
        $tempPaths = @(
            "$env:TEMP\ring-*",
            "$env:TMP\ring-*"
        )
        foreach ($path in $tempPaths) {
            Get-ChildItem $path -ErrorAction SilentlyContinue | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
        }
    }
    catch {
        Write-Host "  Warning: Could not clear all ring cache: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

function Invoke-OptimizedBuild {
    Write-Host "Attempting optimized build..." -ForegroundColor Yellow
    
    # Pokus 1: Ring cache clear + MSVC toolchain
    Write-Host "  Trying MSVC toolchain with clean ring cache..." -ForegroundColor Cyan
    try {
        Clear-RingCrateCache
        $env:RUSTFLAGS = "-C target-cpu=native"
        $env:RING_PREGENERATE_ASM = "1"
        $env:CARGO_BUILD_JOBS = "1"  # Single-threaded pro ring
        
        $result = Start-Process -FilePath "cargo" -ArgumentList "build", "--release", "--target", "x86_64-pc-windows-msvc" -Wait -PassThru -NoNewWindow -RedirectStandardError "build_error.log"
        
        if ($result.ExitCode -eq 0) {
            $msvcPath = "target\x86_64-pc-windows-msvc\release\easyproject-mcp-server.exe"
            if (Test-Path $msvcPath) {
                if (!(Test-Path "target\release")) {
                    New-Item -ItemType Directory -Path "target\release" -Force | Out-Null
                }
                Copy-Item $msvcPath $targetExe -Force
                Write-Host "  MSVC build successful!" -ForegroundColor Green
                return $true
            }
        }
    }
    catch {
        Write-Host "  MSVC build failed: $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    # Pokus 2: Ring workaround s retry
    Write-Host "  Trying ring workaround with retry..." -ForegroundColor Cyan
    for ($i = 1; $i -le 3; $i++) {
        try {
            Write-Host "    Attempt $i/3..." -ForegroundColor Gray
            Clear-RingCrateCache
            Start-Sleep -Seconds 2  # Krátká pauza
            
            $env:RUSTFLAGS = ""
            $env:RING_PREGENERATE_ASM = "1"
            $env:CARGO_BUILD_JOBS = "1"
            
            $result = Start-Process -FilePath "cargo" -ArgumentList "build", "--release" -Wait -PassThru -NoNewWindow -RedirectStandardError "build_error2.log"
            
            if ($result.ExitCode -eq 0 -and (Test-Path $targetExe)) {
                Write-Host "  Ring workaround successful on attempt $i!" -ForegroundColor Green
                return $true
            }
        }
        catch {
            Write-Host "    Attempt $i failed: $($_.Exception.Message)" -ForegroundColor Yellow
        }
    }
    
    # Pokus 3: Build bez optimalizaci a ring dependency
    Write-Host "  Trying minimal build without optimizations..." -ForegroundColor Cyan
    try {
        Clear-RingCrateCache
        $env:RUSTFLAGS = "-C opt-level=1"
        $env:RING_PREGENERATE_ASM = "1"
        $env:CARGO_BUILD_JOBS = "1"
        
        $result = Start-Process -FilePath "cargo" -ArgumentList "build", "--release" -Wait -PassThru -NoNewWindow
        
        if ($result.ExitCode -eq 0 -and (Test-Path $targetExe)) {
            Write-Host "  Minimal build successful!" -ForegroundColor Green
            return $true
        }
    }
    catch {
        Write-Host "  Minimal build failed: $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    Write-Host "  All build attempts failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "=== RING CRATE TROUBLESHOOTING ===" -ForegroundColor Yellow
    Write-Host "The ring crate (cryptography library) is failing to build." -ForegroundColor White
    Write-Host "This is a common issue on Windows. Solutions:" -ForegroundColor White
    Write-Host ""
    Write-Host "1. Install Visual Studio Build Tools:" -ForegroundColor Cyan
    Write-Host "   winget install Microsoft.VisualStudio.2022.BuildTools" -ForegroundColor Gray
    Write-Host ""
    Write-Host "2. Setup build environment:" -ForegroundColor Cyan
    Write-Host "   .\setup-build-tools.ps1" -ForegroundColor Gray
    Write-Host ""
    Write-Host "3. Use existing EXE (if available):" -ForegroundColor Cyan
    Write-Host "   .\deploy.ps1 -SkipBuild" -ForegroundColor Gray
    Write-Host ""
    Write-Host "4. Restart terminal and try again" -ForegroundColor Cyan
    Write-Host ""
    return $false
}

function New-DeploymentPackage($exePath, $sizeMB) {
    Write-Host "Creating deployment package..." -ForegroundColor Yellow
    
    if (!(Test-Path $deployDir)) {
        New-Item -ItemType Directory -Path $deployDir | Out-Null
    }
    
    Copy-Item $exePath $deployExe -Force
    Write-Host "  EXE copied ($sizeMB MB)" -ForegroundColor Cyan
    
    $buildDate = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    $readmeContent = @"
# EasyProject MCP Server - Single-File Deployment

## Ready for Deployment!

Tento EXE soubor ($sizeMB MB) je samostatny a obsahuje vsechny zavislosti.

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
- Velikost: $sizeMB MB
- Datum build: $buildDate
- Single-file deployment
- TLS: Rust-native

## Troubleshooting

Pokud se server nespusti:
1. Zkontrolujte Visual C++ Redistributable 2019+
2. Overte spravnost API klice a URL
3. Zkontrolujte opravneni EXE souboru

Pro ladeni spustte EXE z Command Line.
"@

    $readmeContent | Out-File -FilePath "$deployDir\README.md" -Encoding UTF8
    Write-Host "  README.md updated" -ForegroundColor Cyan
    
    $mcpConfig = @"
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
    
    $mcpConfig | Out-File -FilePath "$deployDir\cursor-mcp-config.json" -Encoding UTF8
    Write-Host "  MCP config template updated" -ForegroundColor Cyan
}

# Hlavni logika
try {
    $buildNeeded = $Force -or !(Test-Path $targetExe)
    $existingSize = Get-FileSizeMB $targetExe
    
    if ($SkipBuild) {
        Write-Host "Skipping build (using existing EXE)" -ForegroundColor Yellow
        $buildNeeded = $false
    }
    
    if ($buildNeeded -and !$SkipBuild) {
        Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
        cargo clean | Out-Null
        
        $buildSuccess = Invoke-OptimizedBuild
        
        if (!$buildSuccess) {
            if (Test-Path $targetExe -and $existingSize -gt 0) {
                Write-Host "Build failed, but using existing EXE ($existingSize MB)" -ForegroundColor Yellow
            } else {
                Write-Host "No EXE available for deployment!" -ForegroundColor Red
                Write-Host "Try: rustup toolchain install stable-x86_64-pc-windows-msvc" -ForegroundColor Yellow
                Write-Host "Or: winget install Microsoft.VisualStudio.2022.BuildTools" -ForegroundColor Yellow
                exit 1
            }
        }
    } elseif (Test-Path $targetExe) {
        Write-Host "Using existing EXE ($existingSize MB)" -ForegroundColor Cyan
    } else {
        Write-Host "No EXE found! Run without -SkipBuild to build first." -ForegroundColor Red
        exit 1
    }
    
    $finalSize = Get-FileSizeMB $targetExe
    New-DeploymentPackage $targetExe $finalSize
    
    Write-Host ""
    Write-Host "Deployment Package Complete!" -ForegroundColor Green
    Write-Host "============================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Contents:" -ForegroundColor Cyan
    Write-Host "  - easyproject-mcp-server.exe ($finalSize MB)" -ForegroundColor White
    Write-Host "  - README.md (deployment guide)" -ForegroundColor White
    Write-Host "  - cursor-mcp-config.json (template)" -ForegroundColor White
    Write-Host ""
    Write-Host "Location: $deployDir" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Ready for deployment!" -ForegroundColor Green

} catch {
    Write-Host "Deployment failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
} 