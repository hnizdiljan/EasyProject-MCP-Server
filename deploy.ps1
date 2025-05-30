# EasyProject MCP Server - Smart Deployment Script
# Pouziti: 
#   .\deploy.ps1              # Normalni build a deploy
#   .\deploy.ps1 -Force       # Vynuti novy build
#   .\deploy.ps1 -SkipBuild   # Pouzije existujici EXE
#   .\deploy.ps1 -CleanCache  # Vycisti cache a skonci

param(
    [switch]$Force,        # Vynuti novy build
    [switch]$SkipBuild,    # Pouzije existujici EXE
    [switch]$CleanCache    # Pouze vycisti cache a skonci
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

function Clear-AllCache {
    Write-Host "Clearing all Rust/Cargo cache..." -ForegroundColor Yellow
    
    # Cargo clean
    Write-Host "  Running cargo clean..." -ForegroundColor Cyan
    cargo clean | Out-Null
    
    # Vymazat target adresář
    if (Test-Path "target") {
        Write-Host "  Removing target directory..." -ForegroundColor Cyan
        Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    # Vyčistit cargo cache
    $cargoHome = $env:CARGO_HOME
    if (-not $cargoHome) {
        $cargoHome = "$env:USERPROFILE\.cargo"
    }
    
    # Ring crate cache
    if (Test-Path "$cargoHome\registry\src") {
        $indexDirs = Get-ChildItem -Path "$cargoHome\registry\src" -Directory -Name "index.crates.io-*" -ErrorAction SilentlyContinue
        foreach ($indexDir in $indexDirs) {
            $fullIndexPath = Join-Path "$cargoHome\registry\src" $indexDir
            $ringDirs = Get-ChildItem -Path $fullIndexPath -Directory -Name "ring-*" -ErrorAction SilentlyContinue
            foreach ($ringDir in $ringDirs) {
                if ($ringDir) {
                    $fullRingPath = Join-Path $fullIndexPath $ringDir.Name
                    Write-Host "  Removing ring cache: $fullRingPath" -ForegroundColor Cyan
                    Remove-Item -Path $fullRingPath -Recurse -Force -ErrorAction SilentlyContinue
                }
            }
        }
    }
    
    # Build cache
    $buildCachePath = "$cargoHome\registry\.cache"
    if (Test-Path $buildCachePath) {
        Write-Host "  Clearing cargo build cache..." -ForegroundColor Cyan
        Remove-Item -Path $buildCachePath -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    # Git cache
    $gitCachePath = "$cargoHome\git"
    if (Test-Path $gitCachePath) {
        Write-Host "  Clearing cargo git cache..." -ForegroundColor Cyan
        Remove-Item -Path $gitCachePath -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    Write-Host "  Cache cleared!" -ForegroundColor Green
}

function Invoke-OptimizedBuild {
    Write-Host "Building with MSVC toolchain..." -ForegroundColor Yellow
    
    try {
        # Důkladnější vyčištění build cache
        Write-Host "  Cleaning build cache thoroughly..." -ForegroundColor Cyan
        
        # Vyčistit cargo cache
        cargo clean | Out-Null
        
        # Vymazat target adresář úplně (řeší problém s ring crate)
        if (Test-Path "target") {
            Write-Host "  Removing target directory..." -ForegroundColor Cyan
            Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 1
        }
        
        # Vyčistit cargo registry cache pro ring (častý problém)
        $cargoHome = $env:CARGO_HOME
        if (-not $cargoHome) {
            $cargoHome = "$env:USERPROFILE\.cargo"
        }
        
        $ringCachePath = "$cargoHome\registry\src\index.crates.io-*\ring-*"
        if (Test-Path $ringCachePath) {
            Write-Host "  Clearing ring crate cache..." -ForegroundColor Cyan
            Remove-Item -Path $ringCachePath -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        # Vyčistit build cache
        $buildCachePath = "$cargoHome\registry\.cache"
        if (Test-Path $buildCachePath) {
            Write-Host "  Clearing cargo build cache..." -ForegroundColor Cyan
            Remove-Item -Path $buildCachePath -Recurse -Force -ErrorAction SilentlyContinue
        }
        
        # Počkat chvíli aby se uvolnily file handles
        Start-Sleep -Seconds 2
        
        # Build s MSVC targetem
        Write-Host "  Building with x86_64-pc-windows-msvc..." -ForegroundColor Cyan
        
        # Nastavit environment pro build
        $env:CARGO_INCREMENTAL = "0"  # Vypnout incremental build
        $env:RUST_BACKTRACE = "1"     # Pro lepší error reporting
        
        $result = Start-Process -FilePath "cargo" -ArgumentList "build", "--release", "--target", "x86_64-pc-windows-msvc" -Wait -PassThru -NoNewWindow -RedirectStandardError "build_error.log"
        
        if ($result.ExitCode -eq 0) {
            $msvcPath = "target\x86_64-pc-windows-msvc\release\easyproject-mcp-server.exe"
            if (Test-Path $msvcPath) {
                # Zkopírujeme do standardního umístění
                if (!(Test-Path "target\release")) {
                    New-Item -ItemType Directory -Path "target\release" -Force | Out-Null
                }
                Copy-Item $msvcPath $targetExe -Force
                
                # Vyčistit environment
                Remove-Item Env:\CARGO_INCREMENTAL -ErrorAction SilentlyContinue
                Remove-Item Env:\RUST_BACKTRACE -ErrorAction SilentlyContinue
                
                Write-Host "  Build successful!" -ForegroundColor Green
                return $true
            }
        }
        
        # Pokud se build nepodařil, zobrazíme chybu
        if (Test-Path "build_error.log") {
            Write-Host "  Build failed. Error log:" -ForegroundColor Red
            Get-Content "build_error.log" | Write-Host -ForegroundColor Yellow
        }
        
        # Vyčistit environment i při chybě
        Remove-Item Env:\CARGO_INCREMENTAL -ErrorAction SilentlyContinue
        Remove-Item Env:\RUST_BACKTRACE -ErrorAction SilentlyContinue
        
    } catch {
        Write-Host "  Build failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Write-Host "  Build failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "=== BUILD TROUBLESHOOTING ===" -ForegroundColor Yellow
    Write-Host "The build process failed. Trying solutions:" -ForegroundColor White
    Write-Host ""
    Write-Host "1. Clear all Rust/Cargo cache:" -ForegroundColor Cyan
    Write-Host "   cargo clean && rmdir /s target" -ForegroundColor Gray
    Write-Host ""
    Write-Host "2. Update Rust toolchain:" -ForegroundColor Cyan
    Write-Host "   rustup update stable" -ForegroundColor Gray
    Write-Host ""
    Write-Host "3. Reinstall MSVC toolchain:" -ForegroundColor Cyan
    Write-Host "   rustup target remove x86_64-pc-windows-msvc" -ForegroundColor Gray
    Write-Host "   rustup target add x86_64-pc-windows-msvc" -ForegroundColor Gray
    Write-Host ""
    Write-Host "4. Use existing EXE (if available):" -ForegroundColor Cyan
    Write-Host "   .\deploy.ps1 -SkipBuild" -ForegroundColor Gray
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

# Pokud je zadán pouze CleanCache, vyčisti a skonči
if ($CleanCache) {
    Clear-AllCache
    Write-Host ""
    Write-Host "Cache cleared successfully!" -ForegroundColor Green
    Write-Host "You can now run: .\deploy.ps1 -Force" -ForegroundColor Yellow
    exit 0
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