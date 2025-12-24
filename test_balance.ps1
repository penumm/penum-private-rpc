# PowerShell script to test eth_getBalance on Penum RPC endpoint
Write-Host "Testing eth_getBalance on Penum RPC endpoint..."

# Test eth_getBalance with a known address
$body = '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x742d35Cc6634C0532925a3b8D4C9db96590b5c8e", "latest"],"id":2}'

try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:8545" -Method Post -ContentType "application/json" -Body $body
    Write-Host "Success! Balance Response: $($response | ConvertTo-Json -Depth 10)"
} 
catch {
    Write-Host "Error: $($_.Exception.Message)"
    if ($_.Exception.Response) {
        Write-Host "Response status: $($_.Exception.Response.StatusCode)"
    }
}