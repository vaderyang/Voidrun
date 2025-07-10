# TypeScript Project in Persistent Sandbox

## 🚀 Complete Solution for Your TypeScript Project

### 1. **Transfer Multiple Files to Sandbox**

```bash
# Create persistent sandbox with your TypeScript project
curl -X POST http://127.0.0.1:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "bun",
    "code": "console.log(\"Starting TypeScript project setup...\");",
    "entry_point": "bun dev",
    "timeout_ms": 300000,
    "memory_limit_mb": 512,
    "mode": "persistent",
    "install_deps": true,
    "dev_server": true,
    "env_vars": {
      "NODE_ENV": "development",
      "PORT": "3000"
    },
    "files": [
      {
        "path": "package.json",
        "content": "{\n  \"name\": \"typescript-project\",\n  \"scripts\": {\n    \"dev\": \"bun run --hot src/index.tsx\",\n    \"build\": \"bun build src/index.tsx --outdir=dist\"\n  },\n  \"dependencies\": {\n    \"react\": \"^18.2.0\",\n    \"react-dom\": \"^18.2.0\"\n  },\n  \"devDependencies\": {\n    \"@types/react\": \"^18.2.0\",\n    \"typescript\": \"^5.0.0\"\n  }\n}"
      },
      {
        "path": "src/App.tsx",
        "content": "import React from \"react\";\n\nconst App: React.FC = () => {\n  return (\n    <div>\n      <h1>Hello TypeScript Sandbox!</h1>\n      <p>Your project is running!</p>\n    </div>\n  );\n};\n\nexport default App;"
      },
      {
        "path": "src/index.tsx",
        "content": "import React from \"react\";\nimport { createRoot } from \"react-dom/client\";\nimport App from \"./App\";\n\nconst container = document.getElementById(\"root\");\nif (container) {\n  const root = createRoot(container);\n  root.render(<App />);\n}"
      },
      {
        "path": "src/styles.css",
        "content": "body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }\nh1 { color: #333; }\np { color: #666; }"
      },
      {
        "path": "public/index.html",
        "content": "<!DOCTYPE html>\n<html>\n<head>\n  <title>TypeScript Sandbox</title>\n  <meta charset=\"UTF-8\">\n</head>\n<body>\n  <div id=\"root\"></div>\n  <script type=\"module\" src=\"/src/index.tsx\"></script>\n</body>\n</html>"
      }
    ]
  }'
```

### 2. **Enhanced API for Persistent Projects**

#### **New Request Fields:**
- `"mode": "persistent"` - Keeps container running
- `"install_deps": true` - Automatically runs `bun install`
- `"dev_server": true` - Starts development server
- `"entry_point": "bun dev"` - Custom startup command

#### **Response includes:**
- `"is_running": true` - Container status
- `"dev_server_url": "http://localhost:3000"` - Access URL
- `"container_id": "abc123"` - For direct container access

### 3. **Workflow for Your TypeScript Project**

```bash
# Step 1: Create persistent sandbox
SANDBOX_RESPONSE=$(curl -s -X POST http://127.0.0.1:8070/sandbox \
  -H "Content-Type: application/json" \
  -d @typescript_project_example.json)

SANDBOX_ID=$(echo "$SANDBOX_RESPONSE" | jq -r '.id')
echo "Sandbox created: $SANDBOX_ID"

# Step 2: The sandbox will automatically:
# - Transfer all your files (app.tsx, page.tsx, styles.css, etc.)
# - Run `bun install` to install dependencies
# - Start the development server with `bun dev`
# - Keep running until you stop it

# Step 3: Check status
curl -s http://127.0.0.1:8070/sandbox/$SANDBOX_ID | jq

# Step 4: Access your running app
# The dev server will be accessible at the provided URL

# Step 5: Stop when done
curl -X DELETE http://127.0.0.1:8070/sandbox/$SANDBOX_ID
```

### 4. **File Structure Support**

Your project structure is fully supported:
```
project/
├── package.json
├── tsconfig.json
├── src/
│   ├── App.tsx
│   ├── Page.tsx
│   ├── index.tsx
│   └── styles.css
├── public/
│   └── index.html
└── components/
    └── Header.tsx
```

### 5. **Advanced Features**

#### **Hot Reload Support:**
```json
{
  "entry_point": "bun run --hot src/index.tsx",
  "dev_server": true
}
```

#### **Custom Build Scripts:**
```json
{
  "files": [
    {
      "path": "package.json",
      "content": "{\n  \"scripts\": {\n    \"dev\": \"bun run --hot src/index.tsx\",\n    \"build\": \"bun build src/index.tsx --outdir=dist\",\n    \"preview\": \"bun run build && bun run dist/index.js\"\n  }\n}"
    }
  ]
}
```

#### **Environment Configuration:**
```json
{
  "env_vars": {
    "NODE_ENV": "development",
    "PORT": "3000",
    "VITE_API_URL": "http://localhost:8070"
  }
}
```

### 6. **New API Endpoints**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `POST /sandbox` | POST | Create persistent sandbox |
| `GET /sandbox/{id}/status` | GET | Check if dev server is running |
| `POST /sandbox/{id}/restart` | POST | Restart dev server |
| `POST /sandbox/{id}/logs` | POST | Get container logs |
| `DELETE /sandbox/{id}` | DELETE | Stop and cleanup |

### 7. **Container Management**

#### **Persistent Container Features:**
- ✅ **Long-running**: Container stays alive until stopped
- ✅ **Port forwarding**: Access dev server from host
- ✅ **Hot reload**: File changes trigger rebuilds
- ✅ **Package installation**: Automatic dependency management
- ✅ **Volume mapping**: Persistent file storage during session

#### **Resource Management:**
```json
{
  "timeout_ms": 3600000,     // 1 hour max runtime
  "memory_limit_mb": 1024,   // 1GB for React/TypeScript
  "dev_server": true,        // Enable port forwarding
  "install_deps": true       // Auto-install packages
}
```

### 8. **Complete Example Script**

```bash
#!/bin/bash
# typescript_sandbox.sh

echo "🚀 Setting up TypeScript project in persistent sandbox..."

# Create sandbox
RESPONSE=$(curl -s -X POST http://127.0.0.1:8070/sandbox \
  -H "Content-Type: application/json" \
  -d @typescript_project_example.json)

SANDBOX_ID=$(echo "$RESPONSE" | jq -r '.id')
echo "✅ Sandbox created: $SANDBOX_ID"

# Wait for setup
echo "⏳ Installing dependencies and starting dev server..."
sleep 30

# Check status
STATUS=$(curl -s http://127.0.0.1:8070/sandbox/$SANDBOX_ID)
echo "📊 Status: $(echo "$STATUS" | jq -r '.status')"

# Get dev server URL
DEV_URL=$(echo "$STATUS" | jq -r '.dev_server_url // "http://localhost:3000"')
echo "🌐 Dev server: $DEV_URL"

echo "🎉 Your TypeScript project is running!"
echo "💡 To stop: curl -X DELETE http://127.0.0.1:8070/sandbox/$SANDBOX_ID"
```

### 9. **Benefits of Persistent Mode**

#### **Development Experience:**
- 🔄 **Hot reload** - Changes reflect immediately
- 📦 **Package management** - Install/update dependencies
- 🎯 **Live debugging** - Real-time error feedback
- 🌐 **Browser access** - Direct URL access to your app

#### **Production-Like Environment:**
- 🐳 **Containerized** - Consistent environment
- 🔒 **Isolated** - No conflicts with host system
- 📊 **Resource controlled** - Memory/CPU limits
- 🧹 **Clean shutdown** - Proper cleanup when done

### 10. **Usage Summary**

```bash
# 1. Transfer your complete TypeScript project
curl -X POST .../sandbox -d @your_project.json

# 2. Access your running development server
# → Automatic bun install
# → Automatic dev server start
# → Hot reload enabled

# 3. Develop with live updates
# → Make changes via API
# → See changes immediately

# 4. Stop when done
curl -X DELETE .../sandbox/{id}
```

This gives you a **complete development environment** for your TypeScript project with **persistent containers**, **automatic dependency installation**, and **live development server** - exactly what you need! 🚀