pub const ADMIN_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sandbox Admin Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', Roboto, sans-serif;
            background: #fafafa;
            color: #1d1d1f;
            line-height: 1.47059;
            min-height: 100vh;
            font-size: 17px;
            font-weight: 400;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }
        
        header {
            background: rgba(255, 255, 255, 0.8);
            backdrop-filter: saturate(180%) blur(20px);
            border-bottom: 1px solid rgba(0, 0, 0, 0.1);
            color: #1d1d1f;
            padding: 1rem 0;
            margin-bottom: 0;
            position: sticky;
            top: 0;
            z-index: 100;
        }
        
        .header-content {
            max-width: 1400px;
            margin: 0 auto;
            padding: 0 20px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .logo {
            font-size: 1.5rem;
            font-weight: 600;
            color: #1d1d1f;
            text-decoration: none;
            letter-spacing: -0.02em;
        }
        
        .nav-tabs {
            display: flex;
            gap: 1rem;
        }
        
        .nav-tab {
            padding: 0.5rem 1rem;
            background: none;
            border: none;
            color: #6e6e73;
            cursor: pointer;
            border-radius: 12px;
            transition: all 0.3s ease;
            font-weight: 400;
            font-size: 14px;
        }
        
        .nav-tab:hover {
            background: rgba(0, 0, 0, 0.04);
            color: #1d1d1f;
        }
        
        .nav-tab.active {
            background: #007aff;
            color: white;
        }
        
        .tab-content {
            display: none;
        }
        
        .tab-content.active {
            display: block;
            padding: 2rem 0;
        }
        
        .status-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }
        
        .status-card {
            background: white;
            padding: 1.5rem;
            border-radius: 18px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.07), 0 1px 3px rgba(0, 0, 0, 0.06);
            border: 1px solid rgba(0, 0, 0, 0.04);
            transition: all 0.3s ease;
        }
        
        .status-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1), 0 3px 6px rgba(0, 0, 0, 0.08);
        }
        
        .status-card h3 {
            color: #1d1d1f;
            margin-bottom: 1rem;
            font-size: 1.1rem;
            font-weight: 600;
            letter-spacing: -0.01em;
        }
        
        .metric {
            display: flex;
            justify-content: space-between;
            margin-bottom: 0.5rem;
            color: #86868b;
            font-size: 14px;
        }
        
        .metric-value {
            font-weight: 600;
            color: #1d1d1f;
        }
        
        .progress-bar {
            width: 100%;
            height: 4px;
            background: #e5e5e7;
            border-radius: 2px;
            overflow: hidden;
            margin-top: 0.5rem;
        }
        
        .progress-fill {
            height: 100%;
            background: #007aff;
            transition: width 0.3s ease;
            border-radius: 2px;
        }
        
        .sandbox-list {
            background: white;
            border: 1px solid rgba(0, 0, 0, 0.04);
            border-radius: 18px;
            overflow: hidden;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.07), 0 1px 3px rgba(0, 0, 0, 0.06);
        }
        
        .sandbox-header {
            background: #f5f5f7;
            color: #1d1d1f;
            padding: 1rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-bottom: 1px solid rgba(0, 0, 0, 0.04);
        }
        
        .refresh-btn {
            background: #007aff;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: all 0.3s ease;
        }
        
        .refresh-btn:hover {
            background: #0056cc;
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);
        }
        
        .sandbox-table {
            width: 100%;
            border-collapse: collapse;
        }
        
        .sandbox-table th,
        .sandbox-table td {
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid #e5e5e7;
            color: #1d1d1f;
            font-size: 14px;
        }
        
        .sandbox-table th {
            background: #f5f5f7;
            font-weight: 600;
            color: #6e6e73;
            font-size: 12px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .status-badge {
            padding: 0.25rem 0.5rem;
            border-radius: 6px;
            font-size: 12px;
            font-weight: 500;
        }
        
        .status-running {
            background: #e3f2fd;
            color: #1565c0;
        }
        
        .status-created {
            background: #f3e5f5;
            color: #7b1fa2;
        }
        
        .status-failed {
            background: #ffebee;
            color: #c62828;
        }
        
        .action-btn {
            padding: 0.25rem 0.5rem;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 12px;
            font-weight: 500;
            margin-right: 0.5rem;
            transition: all 0.3s ease;
        }
        
        .btn-view {
            background: #007aff;
            color: white;
        }
        
        .btn-logs {
            background: #ff9500;
            color: white;
        }
        
        .btn-stop {
            background: #ff3b30;
            color: white;
        }
        
        .btn-proxy {
            background: #5856d6;
            color: white;
        }
        
        .action-btn:hover {
            transform: translateY(-1px);
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
        }

        .proxy-link {
            display: inline-block;
            padding: 0.25rem 0.5rem;
            background: #007aff;
            color: white;
            text-decoration: none;
            border-radius: 6px;
            font-size: 12px;
            font-weight: 500;
            transition: all 0.3s ease;
        }

        .proxy-link:hover {
            background: #0056cc;
            transform: translateY(-1px);
            box-shadow: 0 2px 8px rgba(0, 122, 255, 0.3);
        }

        .proxy-unavailable {
            color: #86868b;
            font-size: 12px;
            font-style: italic;
        }
        
        .log-viewer {
            background: white;
            border: 1px solid rgba(0, 0, 0, 0.04);
            border-radius: 18px;
            overflow: hidden;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.07), 0 1px 3px rgba(0, 0, 0, 0.06);
        }
        
        .log-header {
            background: #f5f5f7;
            color: #1d1d1f;
            padding: 1rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-bottom: 1px solid rgba(0, 0, 0, 0.04);
        }
        
        .log-controls {
            display: flex;
            gap: 1rem;
            align-items: center;
        }
        
        .log-controls select,
        .log-controls input {
            padding: 0.25rem 0.5rem;
            border-radius: 6px;
            border: 1px solid #d1d1d6;
            background: white;
            color: #1d1d1f;
            font-size: 14px;
        }
        
        .log-controls label {
            color: #1d1d1f;
            font-size: 14px;
            font-weight: 500;
        }
        
        .log-content {
            background: #fafafa;
            color: #1d1d1f;
            padding: 1rem;
            font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Courier New', monospace;
            font-size: 13px;
            height: 400px;
            overflow-y: auto;
            border: 1px solid #e5e5e7;
        }
        
        .log-entry {
            margin-bottom: 0.5rem;
        }
        
        .log-timestamp {
            color: #86868b;
        }
        
        .log-level-info {
            color: #007aff;
        }
        
        .log-level-warn {
            color: #ff9500;
        }
        
        .log-level-error {
            color: #ff3b30;
        }
        
        .api-docs {
            background: white;
            border: 1px solid rgba(0, 0, 0, 0.04);
            border-radius: 18px;
            overflow: hidden;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.07), 0 1px 3px rgba(0, 0, 0, 0.06);
        }
        
        .api-header {
            background: #f5f5f7;
            color: #1d1d1f;
            padding: 1rem;
            border-bottom: 1px solid rgba(0, 0, 0, 0.04);
        }
        
        .api-endpoint {
            border-bottom: 1px solid #e5e5e7;
            padding: 1rem;
            color: #1d1d1f;
        }
        
        .api-endpoint:last-child {
            border-bottom: none;
        }
        
        .api-method {
            display: inline-block;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-size: 0.8rem;
            font-weight: bold;
            margin-right: 1rem;
        }
        
        .method-get {
            background: #27ae60;
            color: white;
        }
        
        .method-post {
            background: #3498db;
            color: white;
        }
        
        .method-delete {
            background: #e74c3c;
            color: white;
        }
        
        .api-path {
            font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
            font-weight: 600;
            color: #1d1d1f;
        }
        
        .api-tester {
            background: #f5f5f7;
            padding: 1rem;
            margin-top: 1rem;
            border-radius: 12px;
            border: 1px solid #e5e5e7;
        }
        
        .form-group {
            margin-bottom: 1rem;
        }
        
        .form-group label {
            display: block;
            margin-bottom: 0.5rem;
            font-weight: 600;
            color: #1d1d1f;
            font-size: 14px;
        }
        
        .form-group input,
        .form-group select,
        .form-group textarea {
            width: 100%;
            padding: 0.5rem;
            border: 1px solid #d1d1d6;
            border-radius: 6px;
            font-size: 14px;
            background: white;
            color: #1d1d1f;
            transition: border-color 0.3s ease;
        }
        
        .form-group input:focus,
        .form-group select:focus,
        .form-group textarea:focus {
            outline: none;
            border-color: #007aff;
        }
        
        .form-group textarea {
            height: 100px;
            font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
            resize: vertical;
        }
        
        .test-btn {
            background: #007aff;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: all 0.3s ease;
        }
        
        .test-btn:hover {
            background: #0056cc;
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);
        }
        
        .test-response {
            background: #f5f5f7;
            color: #1d1d1f;
            padding: 1rem;
            border-radius: 12px;
            margin-top: 1rem;
            font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
            font-size: 13px;
            border: 1px solid #e5e5e7;
        }
        
        .loading {
            text-align: center;
            padding: 2rem;
            color: #86868b;
            font-size: 14px;
        }
        
        .error {
            background: #fff2f2;
            color: #d70015;
            padding: 1rem;
            border-radius: 12px;
            margin-bottom: 1rem;
            border: 1px solid #ffc9c9;
            font-size: 14px;
        }
        
        .modal {
            display: none;
            position: fixed;
            z-index: 1000;
            left: 0;
            top: 0;
            width: 100%;
            height: 100%;
            background-color: rgba(0, 0, 0, 0.4);
        }
        
        .modal-content {
            background: white;
            margin: 15% auto;
            padding: 2rem;
            border-radius: 18px;
            width: 80%;
            max-width: 600px;
            color: #1d1d1f;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
            border: 1px solid rgba(0, 0, 0, 0.04);
        }
        
        .close {
            color: #86868b;
            float: right;
            font-size: 28px;
            font-weight: 400;
            cursor: pointer;
            transition: color 0.3s ease;
        }
        
        .close:hover {
            color: #1d1d1f;
        }
        
        @media (max-width: 768px) {
            .header-content {
                flex-direction: column;
                gap: 1rem;
            }
            
            .nav-tabs {
                flex-wrap: wrap;
                justify-content: center;
            }
            
            .status-grid {
                grid-template-columns: 1fr;
            }
            
            .sandbox-table {
                font-size: 0.8rem;
            }
            
            .sandbox-table th,
            .sandbox-table td {
                padding: 0.5rem;
            }
        }
    </style>
</head>
<body>
    <header>
        <div class="header-content">
            <div class="logo">🏗️ Sandbox Admin</div>
            <nav class="nav-tabs">
                <button class="nav-tab active" onclick="showTab('dashboard')">Dashboard</button>
                <button class="nav-tab" onclick="showTab('sandboxes')">Sandboxes</button>
                <button class="nav-tab" onclick="showTab('logs')">Logs</button>
                <button class="nav-tab" onclick="showTab('api-docs')">API Docs</button>
            </nav>
        </div>
    </header>

    <div class="container">
        <!-- Dashboard Tab -->
        <div id="dashboard" class="tab-content active">
            <div class="status-grid">
                <div class="status-card">
                    <h3>System Status</h3>
                    <div class="metric">
                        <span>Uptime:</span>
                        <span class="metric-value" id="uptime">Loading...</span>
                    </div>
                    <div class="metric">
                        <span>Version:</span>
                        <span class="metric-value" id="version">Loading...</span>
                    </div>
                    <div class="metric">
                        <span>Backend:</span>
                        <span class="metric-value" id="backend">Loading...</span>
                    </div>
                </div>
                
                <div class="status-card">
                    <h3>Sandboxes</h3>
                    <div class="metric">
                        <span>Active:</span>
                        <span class="metric-value" id="active-sandboxes">Loading...</span>
                    </div>
                    <div class="metric">
                        <span>Total Created:</span>
                        <span class="metric-value" id="total-sandboxes">Loading...</span>
                    </div>
                </div>
                
                <div class="status-card">
                    <h3>Memory Usage</h3>
                    <div class="metric">
                        <span>Used:</span>
                        <span class="metric-value" id="memory-used">Loading...</span>
                    </div>
                    <div class="progress-bar">
                        <div class="progress-fill" id="memory-progress"></div>
                    </div>
                </div>
                
                <div class="status-card">
                    <h3>CPU Usage</h3>
                    <div class="metric">
                        <span>Used:</span>
                        <span class="metric-value" id="cpu-used">Loading...</span>
                    </div>
                    <div class="progress-bar">
                        <div class="progress-fill" id="cpu-progress"></div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Sandboxes Tab -->
        <div id="sandboxes" class="tab-content">
            <div class="sandbox-list">
                <div class="sandbox-header">
                    <h3>Active Sandboxes</h3>
                    <button class="refresh-btn" onclick="refreshSandboxes()">🔄 Refresh</button>
                </div>
                <table class="sandbox-table">
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Status</th>
                            <th>Runtime</th>
                            <th>Uptime</th>
                            <th>Memory</th>
                            <th>CPU</th>
                            <th>Proxy URL</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody id="sandboxes-tbody">
                        <tr>
                            <td colspan="8" class="loading">Loading sandboxes...</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Logs Tab -->
        <div id="logs" class="tab-content">
            <div class="log-viewer">
                <div class="log-header">
                    <h3>System Logs</h3>
                    <div class="log-controls">
                        <label>
                            Sandbox:
                            <select id="log-sandbox">
                                <option value="">All</option>
                            </select>
                        </label>
                        <label>
                            Lines:
                            <input type="number" id="log-lines" value="100" min="10" max="1000">
                        </label>
                        <button class="refresh-btn" onclick="refreshLogs()">🔄 Refresh</button>
                    </div>
                </div>
                <div class="log-content" id="log-content">
                    <div class="loading">Loading logs...</div>
                </div>
            </div>
        </div>

        <!-- API Docs Tab -->
        <div id="api-docs" class="tab-content">
            <div class="api-docs">
                <div class="api-header">
                    <h3>API Documentation & Tester</h3>
                </div>
                <div id="api-endpoints">
                    <div class="loading">Loading API documentation...</div>
                </div>
            </div>
        </div>
    </div>

    <!-- Modal for sandbox details -->
    <div id="sandbox-modal" class="modal">
        <div class="modal-content">
            <span class="close" onclick="closeSandboxModal()">&times;</span>
            <h2>Sandbox Details</h2>
            <div id="sandbox-details"></div>
        </div>
    </div>

    <script>
        // Global state
        let currentTab = 'dashboard';
        let autoRefresh = null;

        // API base URL
        const API_BASE = '/admin/api';

        // Utility functions
        function formatUptime(seconds) {
            const hours = Math.floor(seconds / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            const secs = seconds % 60;
            return `${hours}h ${minutes}m ${secs}s`;
        }

        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }

        function formatDuration(ms) {
            if (ms < 1000) return ms + 'ms';
            if (ms < 60000) return (ms / 1000).toFixed(1) + 's';
            if (ms < 3600000) return (ms / 60000).toFixed(1) + 'm';
            return (ms / 3600000).toFixed(1) + 'h';
        }

        // Tab management
        function showTab(tabName) {
            // Hide all tabs
            document.querySelectorAll('.tab-content').forEach(tab => {
                tab.classList.remove('active');
            });
            document.querySelectorAll('.nav-tab').forEach(tab => {
                tab.classList.remove('active');
            });

            // Show selected tab
            document.getElementById(tabName).classList.add('active');
            event.target.classList.add('active');
            
            currentTab = tabName;

            // Load tab-specific data
            switch (tabName) {
                case 'dashboard':
                    loadDashboard();
                    break;
                case 'sandboxes':
                    loadSandboxes();
                    break;
                case 'logs':
                    loadLogs();
                    break;
                case 'api-docs':
                    loadApiDocs();
                    break;
            }
        }

        // Dashboard functions
        async function loadDashboard() {
            try {
                const response = await fetch(`${API_BASE}/status`);
                const status = await response.json();
                
                document.getElementById('uptime').textContent = formatUptime(status.uptime);
                document.getElementById('version').textContent = status.version;
                document.getElementById('backend').textContent = status.backend_type;
                document.getElementById('active-sandboxes').textContent = status.active_sandboxes;
                document.getElementById('total-sandboxes').textContent = status.total_sandboxes_created;
                
                // Memory usage
                const memoryUsed = formatBytes(status.memory_usage.used * 1024 * 1024);
                const memoryTotal = formatBytes(status.memory_usage.total * 1024 * 1024);
                document.getElementById('memory-used').textContent = `${memoryUsed} / ${memoryTotal}`;
                document.getElementById('memory-progress').style.width = `${status.memory_usage.percentage}%`;
                
                // CPU usage
                document.getElementById('cpu-used').textContent = `${status.cpu_usage.percentage.toFixed(1)}%`;
                document.getElementById('cpu-progress').style.width = `${status.cpu_usage.percentage}%`;
                
            } catch (error) {
                console.error('Failed to load dashboard:', error);
            }
        }

        // Sandbox functions
        async function loadSandboxes() {
            try {
                const response = await fetch(`${API_BASE}/sandboxes`);
                const sandboxes = await response.json();
                
                const tbody = document.getElementById('sandboxes-tbody');
                tbody.innerHTML = '';
                
                if (sandboxes.length === 0) {
                    tbody.innerHTML = '<tr><td colspan="8" class="loading">No active sandboxes</td></tr>';
                    return;
                }
                
                sandboxes.forEach(sandbox => {
                    const row = document.createElement('tr');
                    row.innerHTML = `
                        <td title="${sandbox.id}">${sandbox.id.substring(0, 8)}...</td>
                        <td><span class="status-badge status-${sandbox.status.toLowerCase()}">${sandbox.status}</span></td>
                        <td>${sandbox.runtime}</td>
                        <td>${formatDuration(sandbox.uptime * 1000)}</td>
                        <td>${sandbox.memory_mb}MB</td>
                        <td>${sandbox.cpu_percentage.toFixed(1)}%</td>
                        <td>
                            ${sandbox.dev_server_url ? `<a href="${sandbox.dev_server_url}" target="_blank" class="proxy-link">🔗 Open App</a>` : '<span class="proxy-unavailable">No dev server</span>'}
                        </td>
                        <td>
                            <button class="action-btn btn-view" onclick="showSandboxDetails('${sandbox.id}')">View</button>
                            <button class="action-btn btn-logs" onclick="showSandboxLogs('${sandbox.id}')">Logs</button>
                            <button class="action-btn btn-stop" onclick="forceStopSandbox('${sandbox.id}')">Stop</button>
                        </td>
                    `;
                    tbody.appendChild(row);
                });
                
                // Update log sandbox dropdown
                updateLogSandboxDropdown(sandboxes);
                
            } catch (error) {
                console.error('Failed to load sandboxes:', error);
            }
        }

        async function refreshSandboxes() {
            await loadSandboxes();
        }

        async function showSandboxDetails(sandboxId) {
            try {
                const response = await fetch(`${API_BASE}/sandboxes/${sandboxId}`);
                const sandbox = await response.json();
                
                const detailsDiv = document.getElementById('sandbox-details');
                detailsDiv.innerHTML = `
                    <p><strong>ID:</strong> ${sandbox.id}</p>
                    <p><strong>Status:</strong> ${sandbox.status}</p>
                    <p><strong>Runtime:</strong> ${sandbox.runtime}</p>
                    <p><strong>Created:</strong> ${new Date(sandbox.created_at).toLocaleString()}</p>
                    <p><strong>Uptime:</strong> ${formatDuration(sandbox.uptime * 1000)}</p>
                    <p><strong>Memory Limit:</strong> ${sandbox.memory_mb}MB</p>
                    <p><strong>Persistent:</strong> ${sandbox.is_persistent ? 'Yes' : 'No'}</p>
                    ${sandbox.dev_server_url ? `<p><strong>Dev Server:</strong> <a href="${sandbox.dev_server_url}" target="_blank">${sandbox.dev_server_url}</a></p>` : ''}
                    ${sandbox.allocated_port ? `<p><strong>Allocated Port:</strong> ${sandbox.allocated_port}</p>` : ''}
                    ${sandbox.container_id ? `<p><strong>Container ID:</strong> ${sandbox.container_id}</p>` : ''}
                `;
                
                document.getElementById('sandbox-modal').style.display = 'block';
            } catch (error) {
                console.error('Failed to load sandbox details:', error);
            }
        }

        function closeSandboxModal() {
            document.getElementById('sandbox-modal').style.display = 'none';
        }

        async function forceStopSandbox(sandboxId) {
            if (!confirm(`Are you sure you want to force stop sandbox ${sandboxId}?`)) {
                return;
            }
            
            try {
                const response = await fetch(`${API_BASE}/sandboxes/${sandboxId}/force-stop`, {
                    method: 'POST'
                });
                const result = await response.json();
                
                if (result.success) {
                    alert('Sandbox stopped successfully');
                    await loadSandboxes();
                } else {
                    alert(`Failed to stop sandbox: ${result.message}`);
                }
            } catch (error) {
                console.error('Failed to stop sandbox:', error);
                alert('Failed to stop sandbox');
            }
        }

        function openProxy(url) {
            window.open(url, '_blank');
        }

        function showSandboxLogs(sandboxId) {
            // Switch to logs tab and filter by sandbox
            showTab('logs');
            document.getElementById('log-sandbox').value = sandboxId;
            loadLogs();
        }

        // Log functions
        async function loadLogs() {
            try {
                const sandboxId = document.getElementById('log-sandbox').value;
                const lines = document.getElementById('log-lines').value;
                
                let url = `${API_BASE}/logs?lines=${lines}`;
                if (sandboxId) {
                    url = `${API_BASE}/sandboxes/${sandboxId}/logs?lines=${lines}`;
                }
                
                const response = await fetch(url);
                const logs = await response.json();
                
                const logContent = document.getElementById('log-content');
                logContent.innerHTML = '';
                
                if (logs.length === 0) {
                    logContent.innerHTML = '<div class="loading">No logs available</div>';
                    return;
                }
                
                logs.forEach(log => {
                    const logEntry = document.createElement('div');
                    logEntry.className = 'log-entry';
                    logEntry.innerHTML = `
                        <span class="log-timestamp">${log.timestamp}</span>
                        <span class="log-level log-level-${log.level.toLowerCase()}">[${log.level}]</span>
                        ${log.sandbox_id ? `<span class="log-sandbox">[${log.sandbox_id}]</span>` : ''}
                        <span class="log-message">${log.message}</span>
                    `;
                    logContent.appendChild(logEntry);
                });
                
                // Scroll to bottom
                logContent.scrollTop = logContent.scrollHeight;
                
            } catch (error) {
                console.error('Failed to load logs:', error);
            }
        }

        function refreshLogs() {
            loadLogs();
        }

        function updateLogSandboxDropdown(sandboxes) {
            const select = document.getElementById('log-sandbox');
            const currentValue = select.value;
            
            select.innerHTML = '<option value="">All</option>';
            
            sandboxes.forEach(sandbox => {
                const option = document.createElement('option');
                option.value = sandbox.id;
                option.textContent = `${sandbox.id.substring(0, 8)}... (${sandbox.runtime})`;
                select.appendChild(option);
            });
            
            select.value = currentValue;
        }

        // API documentation functions
        async function loadApiDocs() {
            try {
                const response = await fetch(`${API_BASE}/docs`);
                const endpoints = await response.json();
                
                const container = document.getElementById('api-endpoints');
                container.innerHTML = '';
                
                endpoints.forEach(endpoint => {
                    const endpointDiv = document.createElement('div');
                    endpointDiv.className = 'api-endpoint';
                    endpointDiv.innerHTML = `
                        <div>
                            <span class="api-method method-${endpoint.method.toLowerCase()}">${endpoint.method}</span>
                            <span class="api-path">${endpoint.path}</span>
                        </div>
                        <p style="margin: 0.5rem 0;">${endpoint.description}</p>
                        
                        ${endpoint.parameters.length > 0 ? `
                            <h4>Parameters:</h4>
                            <ul>
                                ${endpoint.parameters.map(param => `
                                    <li>
                                        <strong>${param.name}</strong> (${param.param_type})
                                        ${param.required ? '<em>required</em>' : '<em>optional</em>'}
                                        - ${param.description}
                                    </li>
                                `).join('')}
                            </ul>
                        ` : ''}
                        
                        <div class="api-tester">
                            <h4>Test this endpoint:</h4>
                            <div class="form-group">
                                <label>Method:</label>
                                <select id="method-${endpoint.method}-${endpoint.path.replace(/[^a-zA-Z0-9]/g, '')}">
                                    <option value="GET" ${endpoint.method === 'GET' ? 'selected' : ''}>GET</option>
                                    <option value="POST" ${endpoint.method === 'POST' ? 'selected' : ''}>POST</option>
                                    <option value="PUT" ${endpoint.method === 'PUT' ? 'selected' : ''}>PUT</option>
                                    <option value="DELETE" ${endpoint.method === 'DELETE' ? 'selected' : ''}>DELETE</option>
                                </select>
                            </div>
                            <div class="form-group">
                                <label>Path:</label>
                                <input type="text" id="path-${endpoint.method}-${endpoint.path.replace(/[^a-zA-Z0-9]/g, '')}" value="${endpoint.path}">
                            </div>
                            <div class="form-group">
                                <label>Request Body (JSON):</label>
                                <textarea id="body-${endpoint.method}-${endpoint.path.replace(/[^a-zA-Z0-9]/g, '')}" placeholder="Enter JSON request body...">${endpoint.example_request || ''}</textarea>
                            </div>
                            <button class="test-btn" onclick="testEndpoint('${endpoint.method}', '${endpoint.path.replace(/[^a-zA-Z0-9]/g, '')}')">Test Endpoint</button>
                            <div id="response-${endpoint.method}-${endpoint.path.replace(/[^a-zA-Z0-9]/g, '')}" class="test-response" style="display: none;"></div>
                        </div>
                    `;
                    container.appendChild(endpointDiv);
                });
                
            } catch (error) {
                console.error('Failed to load API docs:', error);
            }
        }

        async function testEndpoint(method, pathKey) {
            const methodSelect = document.getElementById(`method-${method}-${pathKey}`);
            const pathInput = document.getElementById(`path-${method}-${pathKey}`);
            const bodyTextarea = document.getElementById(`body-${method}-${pathKey}`);
            const responseDiv = document.getElementById(`response-${method}-${pathKey}`);
            
            const requestData = {
                method: methodSelect.value,
                path: pathInput.value,
                body: bodyTextarea.value.trim() || null
            };
            
            try {
                const response = await fetch(`${API_BASE}/test`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify(requestData)
                });
                
                const result = await response.json();
                
                responseDiv.innerHTML = `
                    <div><strong>Status:</strong> ${result.status}</div>
                    <div><strong>Duration:</strong> ${result.duration_ms}ms</div>
                    <div><strong>Response:</strong></div>
                    <pre>${JSON.stringify(JSON.parse(result.body), null, 2)}</pre>
                `;
                responseDiv.style.display = 'block';
                
            } catch (error) {
                console.error('Failed to test endpoint:', error);
                responseDiv.innerHTML = `<div style="color: #e74c3c;"><strong>Error:</strong> ${error.message}</div>`;
                responseDiv.style.display = 'block';
            }
        }

        // Auto-refresh functionality
        function startAutoRefresh() {
            stopAutoRefresh();
            autoRefresh = setInterval(() => {
                switch (currentTab) {
                    case 'dashboard':
                        loadDashboard();
                        break;
                    case 'sandboxes':
                        loadSandboxes();
                        break;
                    case 'logs':
                        loadLogs();
                        break;
                }
            }, 5000); // Refresh every 5 seconds
        }

        function stopAutoRefresh() {
            if (autoRefresh) {
                clearInterval(autoRefresh);
                autoRefresh = null;
            }
        }

        // Initialize the application
        document.addEventListener('DOMContentLoaded', function() {
            loadDashboard();
            startAutoRefresh();
            
            // Close modal when clicking outside
            window.onclick = function(event) {
                const modal = document.getElementById('sandbox-modal');
                if (event.target == modal) {
                    modal.style.display = 'none';
                }
            };
        });

        // Clean up on page unload
        window.addEventListener('beforeunload', function() {
            stopAutoRefresh();
        });
    </script>
</body>
</html>"#;