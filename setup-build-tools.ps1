# Setup Build Tools pro EasyProject MCP Server
# Pouziti: .\setup-build-tools.ps1

Write-Host "Setting up build tools for EasyProject MCP Server" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Green

# Zkontroluj dostupnost rustup
if (!(Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Host "Rustup not found! Please install Rust first:" -ForegroundColor Red
    Write-Host "https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "Current Rust installation:" -ForegroundColor Cyan
rustup show

Write-Host "`nInstalling MSVC toolchain..." -ForegroundColor Yellow
try {
    rustup toolchain install stable-x86_64-pc-windows-msvc
    rustup default stable-x86_64-pc-windows-msvc
    Write-Host "MSVC toolchain installed successfully!" -ForegroundColor Green
} catch {
    Write-Host "Failed to install MSVC toolchain: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nChecking for Visual Studio Build Tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsInstalls = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vsInstalls) {
        Write-Host "Visual Studio Build Tools found!" -ForegroundColor Green
        Write-Host "Installation path: $vsInstalls" -ForegroundColor Cyan
    } else {
        Write-Host "Visual Studio Build Tools not found." -ForegroundColor Yellow
        Write-Host "Installing Visual Studio Build Tools..." -ForegroundColor Yellow
        
        if (Get-Command winget -ErrorAction SilentlyContinue) {
            try {
                winget install Microsoft.VisualStudio.2022.BuildTools --silent
                Write-Host "Build Tools installation started!" -ForegroundColor Green
                Write-Host "Please wait for installation to complete..." -ForegroundColor Yellow
            } catch {
                Write-Host "Failed to install via winget: $($_.Exception.Message)" -ForegroundColor Red
                Write-Host "Please install manually from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Yellow
            }
        } else {
            Write-Host "Winget not available. Please install Build Tools manually:" -ForegroundColor Yellow
            Write-Host "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Cyan
        }
    }
} else {
    Write-Host "Visual Studio Installer not found." -ForegroundColor Yellow
    Write-Host "Please install Visual Studio Build Tools manually:" -ForegroundColor Yellow
    Write-Host "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Cyan
}

Write-Host "`nTesting build environment..." -ForegroundColor Yellow
try {
    $env:RING_PREGENERATE_ASM = "1"
    $testResult = Start-Process -FilePath "cargo" -ArgumentList "check" -Wait -PassThru -NoNewWindow
    
    if ($testResult.ExitCode -eq 0) {
        Write-Host "Build environment is working!" -ForegroundColor Green
        Write-Host "You can now run: .\deploy.ps1" -ForegroundColor Cyan
    } else {
        Write-Host "Build environment test failed." -ForegroundColor Red
        Write-Host "Try restarting your terminal after installing Build Tools." -ForegroundColor Yellow
    }
} catch {
    Write-Host "Build test failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nSetup complete!" -ForegroundColor Green
Write-Host "If you still have build issues, try:" -ForegroundColor Yellow
Write-Host "  1. Restart your terminal" -ForegroundColor White
Write-Host "  2. Run: .\deploy.ps1 -SkipBuild (to use existing EXE)" -ForegroundColor White
Write-Host "  3. Check DEPLOYMENT.md for more options" -ForegroundColor White 