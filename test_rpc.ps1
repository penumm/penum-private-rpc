# PowerShell script to test the Penum RPC endpoint
Write-Host "Testing Penum RPC endpoint..."

# Test eth_blockNumber
$body = '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:8545" -Method Post -ContentType "application/json" -Body $body
    Write-Host "Success! Response: $($response | ConvertTo-Json -Depth 10)"
} 
catch {
    Write-Host "Error: $($_.Exception.Message)"
    if ($_.Exception.Response) {
        Write-Host "Response status: $($_.Exception.Response.StatusCode)"
    }
}