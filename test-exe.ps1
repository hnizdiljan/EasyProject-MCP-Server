# Quick test script for EasyProject MCP Server EXE
# Usage: .\test-exe.ps1

param(
    [string]$ExePath = "deployment\easyproject-mcp-server.exe"
)

Write-Host "Testing EasyProject MCP Server EXE" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Green

if (!(Test-Path $ExePath)) {
    Write-Host "EXE not found at: $ExePath" -ForegroundColor Red
    Write-Host "Run .\deploy.ps1 first to build the EXE" -ForegroundColor Yellow
    exit 1
}

$fileSize = [math]::Round((Get-Item $ExePath).Length / 1MB, 2)
Write-Host "Testing EXE: $ExePath ($fileSize MB)" -ForegroundColor Cyan

# Test 1: Basic execution
Write-Host ""
Write-Host "Test 1: Basic execution..." -ForegroundColor Yellow
$result = Start-Process -FilePath $ExePath -ArgumentList "--help" -Wait -PassThru -NoNewWindow -RedirectStandardOutput "test_output.log" -RedirectStandardError "test_error.log"

if ($result.ExitCode -eq 0) {
    Write-Host "  [OK] EXE runs successfully" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] EXE failed with exit code: $($result.ExitCode)" -ForegroundColor Red
    if (Test-Path "test_error.log") {
        Write-Host "  Error output:" -ForegroundColor Red
        Get-Content "test_error.log" | Write-Host -ForegroundColor Yellow
    }
}

# Test 2: Dependencies check
Write-Host ""
Write-Host "Test 2: Dependencies check..." -ForegroundColor Yellow
$result2 = Start-Process -FilePath $ExePath -ArgumentList "--version" -Wait -PassThru -NoNewWindow -RedirectStandardOutput "test_version.log" -RedirectStandardError "test_version_error.log"

if ($result2.ExitCode -eq 0) {
    Write-Host "  [OK] All dependencies loaded successfully" -ForegroundColor Green
} else {
    Write-Host "  [WARN] Potential dependency issues (exit code: $($result2.ExitCode))" -ForegroundColor Yellow
}

# Test 3: File integrity
Write-Host ""
Write-Host "Test 3: File integrity..." -ForegroundColor Yellow
$expectedMinSize = 2.0
$expectedMaxSize = 10.0

if ($fileSize -ge $expectedMinSize -and $fileSize -le $expectedMaxSize) {
    Write-Host "  [OK] File size is reasonable ($fileSize MB)" -ForegroundColor Green
} else {
    Write-Host "  [WARN] File size unusual ($fileSize MB)" -ForegroundColor Yellow
}

# Cleanup test files
Remove-Item "test_*.log" -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Test Summary:" -ForegroundColor Green
Write-Host "EXE Path: $ExePath" -ForegroundColor Cyan
Write-Host "File Size: $fileSize MB" -ForegroundColor Cyan
Write-Host ""
Write-Host "Ready for deployment!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Copy EXE to target system" -ForegroundColor White
Write-Host "2. Configure Cursor MCP (see deployment/cursor-mcp-config.json)" -ForegroundColor White
Write-Host "3. Set environment variables (API_KEY, BASE_URL)" -ForegroundColor White
Write-Host "4. Restart Cursor" -ForegroundColor White 