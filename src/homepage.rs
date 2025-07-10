use axum::response::Html;

pub async fn homepage() -> Html<String> {
    Html(HOMEPAGE_HTML.to_string())
}

const HOMEPAGE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sandbox Service - Secure Code Execution</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            line-height: 1.6;
            min-height: 100vh;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 0 20px;
        }

        .header {
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-bottom: 1px solid rgba(255, 255, 255, 0.2);
            padding: 1rem 0;
            position: fixed;
            width: 100%;
            top: 0;
            z-index: 1000;
        }

        .header-content {
            display: flex;
            justify-content: space-between;
            align-items: center;
            max-width: 1200px;
            margin: 0 auto;
            padding: 0 20px;
        }

        .logo {
            font-size: 1.5rem;
            font-weight: bold;
            color: white;
            text-decoration: none;
        }

        .nav-links {
            display: flex;
            gap: 2rem;
        }

        .nav-links a {
            color: white;
            text-decoration: none;
            padding: 0.5rem 1rem;
            border-radius: 8px;
            transition: all 0.3s ease;
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255, 255, 255, 0.2);
        }

        .nav-links a:hover {
            background: rgba(255, 255, 255, 0.2);
            transform: translateY(-2px);
        }

        .hero {
            text-align: center;
            padding: 150px 0 100px 0;
            color: white;
        }

        .hero h1 {
            font-size: 3.5rem;
            margin-bottom: 1rem;
            font-weight: 700;
            background: linear-gradient(45deg, #fff, #f0f0f0);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }

        .hero p {
            font-size: 1.3rem;
            margin-bottom: 2rem;
            opacity: 0.9;
            max-width: 600px;
            margin-left: auto;
            margin-right: auto;
        }

        .cta-buttons {
            display: flex;
            gap: 1rem;
            justify-content: center;
            flex-wrap: wrap;
        }

        .cta-button {
            padding: 1rem 2rem;
            border: none;
            border-radius: 50px;
            font-size: 1.1rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            text-decoration: none;
            display: inline-block;
        }

        .cta-primary {
            background: linear-gradient(45deg, #ff6b6b, #ee5a24);
            color: white;
        }

        .cta-secondary {
            background: rgba(255, 255, 255, 0.2);
            color: white;
            border: 2px solid rgba(255, 255, 255, 0.3);
        }

        .cta-button:hover {
            transform: translateY(-3px);
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        }

        .features {
            padding: 80px 0;
            background: rgba(255, 255, 255, 0.05);
            backdrop-filter: blur(10px);
        }

        .features-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-top: 3rem;
        }

        .feature-card {
            background: rgba(255, 255, 255, 0.1);
            padding: 2rem;
            border-radius: 20px;
            text-align: center;
            border: 1px solid rgba(255, 255, 255, 0.2);
            backdrop-filter: blur(10px);
            transition: transform 0.3s ease;
        }

        .feature-card:hover {
            transform: translateY(-5px);
        }

        .feature-icon {
            font-size: 3rem;
            margin-bottom: 1rem;
        }

        .feature-title {
            font-size: 1.5rem;
            font-weight: 600;
            margin-bottom: 1rem;
            color: white;
        }

        .feature-description {
            color: rgba(255, 255, 255, 0.8);
            font-size: 1rem;
        }

        .stats {
            padding: 60px 0;
            text-align: center;
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 2rem;
            margin-top: 2rem;
        }

        .stat-item {
            color: white;
        }

        .stat-number {
            font-size: 3rem;
            font-weight: bold;
            margin-bottom: 0.5rem;
            background: linear-gradient(45deg, #fff, #f0f0f0);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }

        .stat-label {
            font-size: 1.1rem;
            opacity: 0.9;
        }

        .footer {
            background: rgba(0, 0, 0, 0.3);
            color: white;
            text-align: center;
            padding: 2rem 0;
            margin-top: 4rem;
        }

        .footer-content {
            display: flex;
            justify-content: space-between;
            align-items: center;
            max-width: 1200px;
            margin: 0 auto;
            padding: 0 20px;
        }

        .footer-links {
            display: flex;
            gap: 2rem;
        }

        .footer-links a {
            color: rgba(255, 255, 255, 0.8);
            text-decoration: none;
            transition: color 0.3s ease;
        }

        .footer-links a:hover {
            color: white;
        }

        .api-example {
            background: rgba(0, 0, 0, 0.3);
            padding: 2rem;
            border-radius: 15px;
            margin: 2rem 0;
            font-family: 'Courier New', monospace;
            color: #e2e8f0;
            overflow-x: auto;
        }

        .api-example pre {
            margin: 0;
            font-size: 0.9rem;
            line-height: 1.4;
        }

        .section-title {
            text-align: center;
            font-size: 2.5rem;
            font-weight: 600;
            margin-bottom: 1rem;
            color: white;
        }

        .section-subtitle {
            text-align: center;
            font-size: 1.2rem;
            color: rgba(255, 255, 255, 0.8);
            margin-bottom: 2rem;
        }

        @media (max-width: 768px) {
            .hero h1 {
                font-size: 2.5rem;
            }
            
            .hero p {
                font-size: 1.1rem;
            }
            
            .cta-buttons {
                flex-direction: column;
                align-items: center;
            }
            
            .features-grid {
                grid-template-columns: 1fr;
            }
            
            .footer-content {
                flex-direction: column;
                gap: 1rem;
            }
            
            .footer-links {
                flex-wrap: wrap;
                justify-content: center;
            }
        }
    </style>
</head>
<body>
    <header class="header">
        <div class="header-content">
            <a href="/" class="logo">🏗️ Sandbox Service</a>
            <nav class="nav-links">
                <a href="/admin">Admin Dashboard</a>
                <a href="#features">Features</a>
                <a href="#api">API</a>
            </nav>
        </div>
    </header>

    <section class="hero">
        <div class="container">
            <h1>Serverless Sandbox Platform</h1>
            <p>Deploy functions, run TypeScript, Node.js, and Bun code with hot reload, file updates, and automatic scaling in isolated sandboxes.</p>
            <div class="cta-buttons">
                <a href="/admin" class="cta-button cta-primary">Admin Dashboard</a>
                <a href="#api" class="cta-button cta-secondary">Try FaaS API</a>
            </div>
        </div>
    </section>

    <section class="features" id="features">
        <div class="container">
            <h2 class="section-title">Key Features</h2>
            <p class="section-subtitle">Built for security, performance, and developer experience</p>
            
            <div class="features-grid">
                <div class="feature-card">
                    <div class="feature-icon">🔒</div>
                    <h3 class="feature-title">Secure Isolation</h3>
                    <p class="feature-description">Each sandbox runs in complete isolation with Docker or NsJail backends, preventing code from affecting the host system.</p>
                </div>
                
                <div class="feature-card">
                    <div class="feature-icon">⚡</div>
                    <h3 class="feature-title">Multiple Runtimes</h3>
                    <p class="feature-description">Support for TypeScript, Node.js, and Bun with automatic dependency management and fast startup times.</p>
                </div>
                
                <div class="feature-card">
                    <div class="feature-icon">🖥️</div>
                    <h3 class="feature-title">Development Server</h3>
                    <p class="feature-description">Run web applications with built-in development server support and automatic port forwarding.</p>
                </div>
                
                <div class="feature-card">
                    <div class="feature-icon">📊</div>
                    <h3 class="feature-title">Resource Monitoring</h3>
                    <p class="feature-description">Real-time monitoring of CPU, memory, and disk usage with configurable limits and alerts.</p>
                </div>
                
                <div class="feature-card">
                    <div class="feature-icon">🔄</div>
                    <h3 class="feature-title">Persistent Mode</h3>
                    <p class="feature-description">Long-running sandboxes for interactive development and testing environments.</p>
                </div>
                
                <div class="feature-card">
                    <div class="feature-icon">🚀</div>
                    <h3 class="feature-title">FaaS/Serverless</h3>
                    <p class="feature-description">Deploy functions with automatic lifecycle management, hot reload, and unique URLs.</p>
                </div>
                
                <div class="feature-card">
                    <div class="feature-icon">🔄</div>
                    <h3 class="feature-title">Live File Updates</h3>
                    <p class="feature-description">Update code in running deployments with instant hot reload support.</p>
                </div>
                
                <div class="feature-card">
                    <div class="feature-icon">🛠️</div>
                    <h3 class="feature-title">Admin Dashboard</h3>
                    <p class="feature-description">Complete management interface with real-time stats, logs, and sandbox control.</p>
                </div>
            </div>
        </div>
    </section>

    <section class="stats">
        <div class="container">
            <h2 class="section-title">Performance & Reliability</h2>
            <div class="stats-grid">
                <div class="stat-item">
                    <div class="stat-number">&lt; 1s</div>
                    <div class="stat-label">Startup Time</div>
                </div>
                <div class="stat-item">
                    <div class="stat-number">99.9%</div>
                    <div class="stat-label">Uptime</div>
                </div>
                <div class="stat-item">
                    <div class="stat-number">1000+</div>
                    <div class="stat-label">Concurrent Sandboxes</div>
                </div>
                <div class="stat-item">
                    <div class="stat-number">256MB</div>
                    <div class="stat-label">Default Memory Limit</div>
                </div>
            </div>
        </div>
    </section>

    <section class="features" id="api">
        <div class="container">
            <h2 class="section-title">Simple REST API</h2>
            <p class="section-subtitle">Get started with just a few HTTP requests</p>
            
            <div class="api-example">
                <pre>
# Deploy a serverless function
POST /faas/deploy
{
  "runtime": "bun",
  "files": [
    {
      "path": "package.json",
      "content": "{\"scripts\": {\"dev\": \"bun run --hot index.ts\"}}"
    },
    {
      "path": "index.ts", 
      "content": "import express from 'express'; const app = express(); app.get('/', (req, res) => res.json({message: 'Hello FaaS!'})); app.listen(3000);"
    }
  ],
  "entry_point": "bun dev"
}

# Access your deployed service
GET /faas/{deployment_id}/

# Update files with hot reload
PUT /faas/deployments/{deployment_id}/files
{
  "files": [{"path": "index.ts", "content": "..."}],
  "restart_dev_server": true
}
</pre>
            </div>
        </div>
    </section>

    <footer class="footer">
        <div class="footer-content">
            <div>&copy; 2025 Sandbox Service. Built with Rust & Axum.</div>
            <div class="footer-links">
                <a href="/health">Health Check</a>
                <a href="/admin">Admin</a>
                <a href="#api">API Docs</a>
            </div>
        </div>
    </footer>
</body>
</html>"##;