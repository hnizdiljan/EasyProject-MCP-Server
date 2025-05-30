# Test MCP komunikace
$env:EASYPROJECT_API_KEY="test-api-key"
$env:EASYPROJECT_BASE_URL="https://demo.easyproject.com"

# Inicializace
$initialize = @{
    jsonrpc = "2.0"
    id = 1
    method = "initialize"
    params = @{
        protocolVersion = "2024-11-05"
        capabilities = @{
            experimental = @{}
            sampling = @{}
        }
        clientInfo = @{
            name = "test-client"
            version = "1.0.0"
        }
    }
} | ConvertTo-Json -Depth 5

Write-Host "Posílám initialize request:"
Write-Host $initialize

# Seznam tools
$listTools = @{
    jsonrpc = "2.0"
    id = 2
    method = "tools/list"
    params = @{}
} | ConvertTo-Json

Write-Host ""
Write-Host "Posílám tools/list request:"
Write-Host $listTools

# Testování přes echo
Write-Host ""
Write-Host "Test komunikace:"
($initialize + "`n" + $listTools) | .\target\release\easyproject-mcp-server.exe 