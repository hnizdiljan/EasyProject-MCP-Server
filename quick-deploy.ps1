# Quick Deployment Script - pouzije existujici EXE
# Pouziti: .\quick-deploy.ps1

Write-Host "Quick Deployment - EasyProject MCP Server" -ForegroundColor Green

$deployDir = "deployment"
$targetExe = "target\release\easyproject-mcp-server.exe"

if (!(Test-Path $targetExe)) {
    Write-Host "EXE not found at: $targetExe" -ForegroundColor Red
    Write-Host "Run 'cargo build --release' first" -ForegroundColor Yellow
    exit 1
}

$fileSize = (Get-Item $targetExe).Length
$fileSizeMB = [math]::Round($fileSize / 1MB, 2)

Write-Host "Found EXE: $fileSizeMB MB" -ForegroundColor Cyan

if (!(Test-Path $deployDir)) {
    New-Item -ItemType Directory -Path $deployDir | Out-Null
}

Copy-Item $targetExe "$deployDir\easyproject-mcp-server.exe" -Force

Write-Host "Deployment complete!" -ForegroundColor Green
Write-Host "EXE copied to: $deployDir\easyproject-mcp-server.exe" -ForegroundColor Yellow
Write-Host "Size: $fileSizeMB MB" -ForegroundColor Yellow 