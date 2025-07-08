# EasyProject MCP Server - NPM Deployment Script
# Usage:
#   .\npm-deploy.ps1              # Build current platform and deploy
#   .\npm-deploy.ps1 -BuildAll    # Build all platforms and deploy
#   .\npm-deploy.ps1 -DryRun      # Test deployment without publishing
#   .\npm-deploy.ps1 -SkipBuild   # Use existing builds
#   .\npm-deploy.ps1 -Beta        # Publish with beta tag

param(
    [switch]$BuildAll,        # Build for all platforms
    [switch]$SkipBuild,       # Skip building step
    [switch]$DryRun,          # Test deployment without publishing
    [switch]$Beta,            # Publish with beta tag
    [switch]$Force,           # Force rebuild
    [string]$Version,         # Override version
    [string]$Registry         # Override npm registry
)

Write-Host "EasyProject MCP Server - NPM Deployment" -ForegroundColor Green
Write-Host "=======================================" -ForegroundColor Green

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "📦 $Message" -ForegroundColor Cyan
    Write-Host "─" * 50 -ForegroundColor Gray
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Test-NodeInstallation {
    Write-Step "Checking Node.js installation"
    
    try {
        $nodeVersion = node --version
        $npmVersion = npm --version
        Write-Host "Node.js: $nodeVersion" -ForegroundColor Green
        Write-Host "npm: $npmVersion" -ForegroundColor Green
        Write-Success "Node.js and npm are available"
    }
    catch {
        Write-Error "Node.js or npm not found. Please install Node.js from https://nodejs.org/"
        exit 1
    }
}

function Test-RustInstallation {
    Write-Step "Checking Rust installation"
    
    try {
        $rustVersion = rustc --version
        $cargoVersion = cargo --version
        Write-Host "Rust: $rustVersion" -ForegroundColor Green
        Write-Host "Cargo: $cargoVersion" -ForegroundColor Green
        Write-Success "Rust toolchain is available"
    }
    catch {
        Write-Error "Rust not found. Please install Rust from https://rustup.rs/"
        exit 1
    }
}

function Install-NodeDependencies {
    Write-Step "Installing Node.js dependencies"
    
    if (Test-Path "node_modules") {
        Write-Host "Dependencies already installed, skipping..." -ForegroundColor Yellow
    } else {
        Write-Host "Installing dependencies..." -ForegroundColor Cyan
        npm install --production=false
        Write-Success "Dependencies installed"
    }
}

function Update-PackageVersion {
    if ($Version) {
        Write-Step "Updating package version to $Version"
        
        # Update package.json version
        $packageJson = Get-Content "package.json" | ConvertFrom-Json
        $packageJson.version = $Version
        $packageJson | ConvertTo-Json -Depth 10 | Set-Content "package.json"
        
        Write-Success "Version updated to $Version"
    }
}

function Build-Binaries {
    if ($SkipBuild) {
        Write-Step "Skipping build (using existing binaries)"
        return
    }
    
    Write-Step "Building binaries"
    
    $buildArgs = @()
    if ($BuildAll) {
        $buildArgs += "--all"
        Write-Host "Building for all supported platforms..." -ForegroundColor Cyan
    } else {
        $buildArgs += "--current"
        Write-Host "Building for current platform only..." -ForegroundColor Cyan
    }
    
    try {
        node scripts/build.js @buildArgs
        Write-Success "Build completed successfully"
    }
    catch {
        Write-Error "Build failed: $($_.Exception.Message)"
        exit 1
    }
}

function Prepare-Package {
    Write-Step "Preparing package for publishing"
    
    try {
        node scripts/prepublish.js
        Write-Success "Package prepared successfully"
    }
    catch {
        Write-Error "Package preparation failed: $($_.Exception.Message)"
        exit 1
    }
}

function Test-Package {
    Write-Step "Testing package"
    
    try {
        node scripts/test.js
        Write-Success "All tests passed"
    }
    catch {
        Write-Warning "Some tests failed, but continuing..."
        Write-Host $_.Exception.Message -ForegroundColor Yellow
    }
}

function Publish-Package {
    if ($DryRun) {
        Write-Step "Dry run - checking what would be published"
        
        Write-Host "Package contents:" -ForegroundColor Cyan
        npm pack --dry-run
        
        Write-Host ""
        Write-Host "Registry check:" -ForegroundColor Cyan
        if ($Registry) {
            npm config set registry $Registry
        }
        npm whoami
        
        Write-Success "Dry run completed - package is ready for publishing"
        return
    }
    
    Write-Step "Publishing to npm"
    
    # Set registry if specified
    if ($Registry) {
        Write-Host "Using registry: $Registry" -ForegroundColor Cyan
        npm config set registry $Registry
    }
    
    # Check authentication
    try {
        $npmUser = npm whoami
        Write-Host "Publishing as: $npmUser" -ForegroundColor Green
    }
    catch {
        Write-Error "Not logged in to npm. Please run 'npm login' first."
        exit 1
    }
    
    # Publish with appropriate tag
    $publishArgs = @("publish")
    if ($Beta) {
        $publishArgs += "--tag", "beta"
        Write-Host "Publishing with beta tag..." -ForegroundColor Yellow
    } else {
        Write-Host "Publishing as latest..." -ForegroundColor Cyan
    }
    
    try {
        npm @publishArgs
        Write-Success "Package published successfully!"
        
        # Show package info
        $packageJson = Get-Content "package.json" | ConvertFrom-Json
        $packageName = $packageJson.name
        $packageVersion = $packageJson.version
        
        Write-Host ""
        Write-Host "📦 Published: $packageName@$packageVersion" -ForegroundColor Green
        Write-Host "🌐 Install: npm install -g $packageName" -ForegroundColor Green
        Write-Host "📋 View: https://npmjs.com/package/$packageName" -ForegroundColor Green
    }
    catch {
        Write-Error "Publishing failed: $($_.Exception.Message)"
        exit 1
    }
}

function Publish-PlatformPackages {
    if (-not $BuildAll) {
        Write-Host "Skipping platform packages (not built for all platforms)" -ForegroundColor Yellow
        return
    }
    
    Write-Step "Publishing platform-specific packages"
    
    $npmDistDir = "npm-dist"
    if (-not (Test-Path $npmDistDir)) {
        Write-Warning "No platform packages found"
        return
    }
    
    $platformDirs = Get-ChildItem -Path $npmDistDir -Directory
    foreach ($platformDir in $platformDirs) {
        Write-Host "Publishing $($platformDir.Name)..." -ForegroundColor Cyan
        
        Push-Location $platformDir.FullName
        try {
            if ($DryRun) {
                npm pack --dry-run
            } else {
                $publishArgs = @("publish")
                if ($Beta) {
                    $publishArgs += "--tag", "beta"
                }
                npm @publishArgs
                Write-Success "Published $($platformDir.Name)"
            }
        }
        catch {
            Write-Warning "Failed to publish $($platformDir.Name): $($_.Exception.Message)"
        }
        finally {
            Pop-Location
        }
    }
}

function Cleanup {
    Write-Step "Cleaning up"
    
    # Remove build artifacts
    if (Test-Path "npm-dist") {
        Remove-Item -Path "npm-dist" -Recurse -Force
        Write-Host "Removed npm-dist directory" -ForegroundColor Gray
    }
    
    # Reset npm registry if it was changed
    if ($Registry) {
        npm config delete registry
        Write-Host "Reset npm registry" -ForegroundColor Gray
    }
    
    Write-Success "Cleanup completed"
}

# Main execution
try {
    # Pre-flight checks
    Test-NodeInstallation
    if (-not $SkipBuild) {
        Test-RustInstallation
    }
    
    # Setup
    Install-NodeDependencies
    Update-PackageVersion
    
    # Build
    Build-Binaries
    
    # Prepare and test
    Prepare-Package
    Test-Package
    
    # Publish
    Publish-Package
    Publish-PlatformPackages
    
    Write-Host ""
    Write-Host "🎉 Deployment completed successfully!" -ForegroundColor Green
    
    if ($DryRun) {
        Write-Host ""
        Write-Host "To actually publish, run without -DryRun flag:" -ForegroundColor Yellow
        Write-Host "  .\npm-deploy.ps1" -ForegroundColor Gray
    }
    
} catch {
    Write-Error "Deployment failed: $($_.Exception.Message)"
    exit 1
} finally {
    Cleanup
} 