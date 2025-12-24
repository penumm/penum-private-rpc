use warp::Filter;

pub async fn start_ui_server(port: u16, rpc_port: u16) -> anyhow::Result<()> {
    let html = move || {
        warp::any().map(move || {
            warp::reply::html(format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Penum RPC - Privacy-Preserving Ethereum Gateway</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }}
        .container {{
            background: white;
            border-radius: 24px;
            padding: 48px;
            max-width: 600px;
            width: 100%;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }}
        .status-indicator {{
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 32px;
        }}
        .status-dot {{
            width: 16px;
            height: 16px;
            border-radius: 50%;
            background: #10b981;
            margin-right: 12px;
            animation: pulse 2s infinite;
        }}
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}
        h1 {{
            font-size: 32px;
            font-weight: 700;
            color: #1f2937;
            margin-bottom: 8px;
            text-align: center;
        }}
        .subtitle {{
            color: #6b7280;
            text-align: center;
            margin-bottom: 32px;
            font-size: 16px;
        }}
        .info-card {{
            background: #f9fafb;
            border-radius: 12px;
            padding: 24px;
            margin-bottom: 16px;
        }}
        .info-label {{
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            color: #6b7280;
            margin-bottom: 8px;
            letter-spacing: 0.5px;
        }}
        .info-value {{
            font-size: 18px;
            font-weight: 600;
            color: #1f2937;
            font-family: 'Monaco', monospace;
            word-break: break-all;
        }}
        .features {{
            margin-top: 32px;
        }}
        .feature {{
            display: flex;
            align-items: start;
            margin-bottom: 16px;
        }}
        .feature-icon {{
            width: 24px;
            height: 24px;
            background: #667eea;
            border-radius: 8px;
            margin-right: 12px;
            flex-shrink: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            font-weight: bold;
        }}
        .feature-text {{
            color: #4b5563;
            font-size: 14px;
            line-height: 1.6;
        }}
        .warning {{
            background: #fef3c7;
            border-left: 4px solid #f59e0b;
            padding: 16px;
            border-radius: 8px;
            margin-top: 24px;
        }}
        .warning-title {{
            font-weight: 600;
            color: #92400e;
            margin-bottom: 4px;
        }}
        .warning-text {{
            color: #78350f;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="status-indicator">
            <div class="status-dot"></div>
            <span style="font-weight: 600; color: #10b981; font-size: 18px;">Penum RPC Running</span>
        </div>
        
        <h1>🔒 Penum RPC</h1>
        <p class="subtitle">Privacy-Preserving Ethereum Gateway</p>
        
        <div class="info-card">
            <div class="info-label">RPC Endpoint</div>
            <div class="info-value">http://127.0.0.1:{}</div>
        </div>
        
        <div class="info-card">
            <div class="info-label">Connection Health</div>
            <div class="info-value" style="color: #10b981;">Connected</div>
        </div>
        
        <div class="features">
            <div class="feature">
                <div class="feature-icon">🛡️</div>
                <div class="feature-text">
                    <strong>IP Privacy:</strong> RPC provider only sees gateway IP, not yours
                </div>
            </div>
            <div class="feature">
                <div class="feature-icon">🔐</div>
                <div class="feature-text">
                    <strong>Fixed-Size Packets:</strong> All traffic encrypted in 1024-byte packets
                </div>
            </div>
            <div class="feature">
                <div class="feature-icon">🚫</div>
                <div class="feature-text">
                    <strong>Zero Logging:</strong> No wallet addresses or transaction data logged
                </div>
            </div>
            <div class="feature">
                <div class="feature-icon">⚡</div>
                <div class="feature-text">
                    <strong>Ephemeral Keys:</strong> New keys for every connection, no state stored
                </div>
            </div>
        </div>
        
        <div class="warning">
            <div class="warning-title">⚠️ Configuration Required</div>
            <div class="warning-text">
                Point MetaMask to <code>http://127.0.0.1:{}</code> to route traffic through Penum.
            </div>
        </div>
    </div>
</body>
</html>"#,
                rpc_port, rpc_port
            ))
        })
    };

    println!("🎨 Penum UI available at http://127.0.0.1:{}", port);

    warp::serve(html()).run(([127, 0, 0, 1], port)).await;

    Ok(())
}
