Write-Host "🔨 Building Penum Private RPC..." -ForegroundColor Cyan

function Build-Component {
    param($Path, $Name)
    
    Write-Host "`n📦 Building $Name..." -ForegroundColor Yellow
    Push-Location $Path
    
    cargo clean | Out-Null
    Start-Sleep -Seconds 2
    
    cargo build --release 2>&1 | Out-Null
    
    Pop-Location
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✅ $Name built successfully" -ForegroundColor Green
        return $true
    }
    else {
        Write-Host "  ❌ Failed to build $Name" -ForegroundColor Red
        return $false
    }
}

$gatewayOk = Build-Component "penum-rpc-gateway" "Gateway"
$clientOk = Build-Component "penum-rpc-client" "Client"

Write-Host "`n═══════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Build Summary" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan

if ($gatewayOk) {
    Write-Host "Gateway:  ✅ READY" -ForegroundColor Green
}
else {
    Write-Host "Gateway:  ❌ FAILED" -ForegroundColor Red
}

if ($clientOk) {
    Write-Host "Client:   ✅ READY" -ForegroundColor Green
}
else {
    Write-Host "Client:   ❌ FAILED" -ForegroundColor Red
}

Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan

if ($gatewayOk -and $clientOk) {
    Write-Host "`n✨ Build complete! Run './run.ps1' to start" -ForegroundColor Green
    exit 0
}
else {
    Write-Host "`n⚠️  Build incomplete. See errors above." -ForegroundColor Yellow
    exit 1
}
