#!/usr/bin/env pwsh
# Run Penum Private RPC system
# Starts gateway and client in separate windows

Write-Host "🚀 Starting Penum Private RPC..." -ForegroundColor Cyan

# Check if binaries exist
$gatewayBin = "penum-rpc-gateway\target\release\penum-rpc-gateway.exe"
$clientBin = "penum-rpc-client\target\release\penum-rpc-client.exe"

if (-not (Test-Path $gatewayBin)) {
    Write-Host "❌ Gateway binary not found. Run './build.ps1' first" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $clientBin)) {
    Write-Host "❌ Client binary not found. Run './build.ps1' first" -ForegroundColor Red
    exit 1
}

# Check config files
if (-not (Test-Path "penum-rpc-gateway\config.json")) {
    Write-Host "⚠️  Gateway config not found, creating from example..." -ForegroundColor Yellow
    Copy-Item "penum-rpc-gateway\config.example.json" "penum-rpc-gateway\config.json"
    Write-Host "📝 IMPORTANT: Edit penum-rpc-gateway\config.json and add your RPC provider API key!" -ForegroundColor Cyan
}

if (-not (Test-Path "penum-rpc-client\config.json")) {
    Write-Host "⚠️  Client config not found, creating from example..." -ForegroundColor Yellow
    Copy-Item "penum-rpc-client\config.example.json" "penum-rpc-client\config.json"
}

# Start gateway in new window
Write-Host "`n🌐 Starting Gateway..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$PWD\penum-rpc-gateway'; .\target\release\penum-rpc-gateway.exe"

Start-Sleep -Seconds 2

# Start client in new window  
Write-Host "🔒 Starting Client..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$PWD\penum-rpc-client'; .\target\release\penum-rpc-client.exe"

Start-Sleep -Seconds 2

Write-Host "`n✨ Penum RPC is running!" -ForegroundColor Green
Write-Host "`n📋 Next steps:" -ForegroundColor Cyan
Write-Host "  1. Open http://127.0.0.1:8546 to view UI"
Write-Host "  2. Configure MetaMask to use http://127.0.0.1:8545"
Write-Host "  3. Send test transaction"
Write-Host "`n💡 Press Ctrl+C in each window to stop components"
