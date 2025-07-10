# 🚀 Complete TypeScript Project in Persistent Sandbox

## **Your Exact Workflow - Step by Step**

### **1. Project Structure You Can Transfer**
```
my-typescript-project/
├── package.json          # Dependencies & scripts
├── tsconfig.json         # TypeScript configuration
├── src/
│   ├── index.tsx        # Entry point
│   ├── App.tsx          # Main component
│   ├── Page.tsx         # Page component
│   └── styles.css       # Styles
└── public/
    └── index.html       # HTML template
```

### **2. Complete Request Format**
```json
{
  "runtime": "bun",
  "code": "console.log('🚀 Setting up TypeScript project...');",
  "entry_point": "bun dev",
  "timeout_ms": 3600000,
  "memory_limit_mb": 1024,
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
      "content": "{\n  \"name\": \"my-typescript-project\",\n  \"scripts\": {\n    \"dev\": \"bun run --hot src/index.tsx\",\n    \"build\": \"bun build src/index.tsx --outdir=dist\"\n  },\n  \"dependencies\": {\n    \"react\": \"^18.2.0\",\n    \"react-dom\": \"^18.2.0\"\n  },\n  \"devDependencies\": {\n    \"@types/react\": \"^18.2.0\",\n    \"typescript\": \"^5.0.0\"\n  }\n}"
    },
    {
      "path": "src/App.tsx",
      "content": "import React from 'react';\n\nconst App: React.FC = () => {\n  return (\n    <div>\n      <h1>Hello TypeScript!</h1>\n      <p>Your project is running!</p>\n    </div>\n  );\n};\n\nexport default App;"
    },
    {
      "path": "src/Page.tsx",
      "content": "import React from 'react';\n\nconst Page: React.FC = () => {\n  return <div>This is your page component!</div>;\n};\n\nexport default Page;"
    },
    {
      "path": "src/styles.css",
      "content": "body { font-family: Arial, sans-serif; padding: 20px; }\nh1 { color: #333; }"
    }
  ]
}
```

### **3. Automated Workflow**

#### **Step 1: Create Persistent Sandbox**
```bash
curl -X POST http://127.0.0.1:8070/sandbox \
  -H "Content-Type: application/json" \
  -d @my_typescript_project.json
```

#### **What Happens Automatically:**
1. 📁 **File Transfer**: All your files (App.tsx, Page.tsx, styles.css) are transferred
2. 🐳 **Container Creation**: Persistent Docker container is created
3. 📦 **Dependency Installation**: `bun install` runs automatically
4. 🚀 **Dev Server Start**: `bun dev` starts with hot reload
5. 🔄 **Keeps Running**: Container stays alive until you stop it

#### **Step 2: Check Status**
```bash
curl http://127.0.0.1:8070/sandbox/{sandbox_id}
```

#### **Step 3: Access Your App**
- Your TypeScript app runs at `http://localhost:3000`
- Hot reload is active
- All changes are immediately reflected

#### **Step 4: Stop When Done**
```bash
curl -X DELETE http://127.0.0.1:8070/sandbox/{sandbox_id}
```

### **4. Key Features for Your Use Case**

#### **✅ Multi-File Support**
- Transfer entire project structure in one request
- Supports nested directories (`src/components/Header.tsx`)
- Maintains file permissions and structure

#### **✅ Persistent Container**
- Container stays running (not ephemeral)
- Development server keeps running
- Hot reload works continuously
- No need to restart between changes

#### **✅ Automatic Setup**
- `bun install` runs automatically
- Dependencies are installed
- Dev server starts automatically
- TypeScript compilation works

#### **✅ Development Experience**
- Hot reload with file watching
- TypeScript type checking
- React development server
- Console output and error reporting

### **5. Management Commands**

I've created a management script for you:

```bash
# Create new project
./manage_typescript_project.sh create

# Check status
./manage_typescript_project.sh status

# View logs
./manage_typescript_project.sh logs

# Stop project
./manage_typescript_project.sh stop
```

### **6. Real Example Session**

```bash
# 1. Start service
./target/release/sandbox-service --port 8070 &

# 2. Create your project
curl -X POST http://127.0.0.1:8070/sandbox \
  -H "Content-Type: application/json" \
  -d @my_typescript_project.json

# Response:
# {
#   "id": "abc123",
#   "status": "created",
#   "runtime": "bun"
# }

# 3. Check status (after ~30 seconds)
curl http://127.0.0.1:8070/sandbox/abc123

# Response:
# {
#   "id": "abc123", 
#   "status": "DevServer",
#   "dev_server_url": "http://localhost:3000"
# }

# 4. Access your app at http://localhost:3000
# 5. Make changes via API or file updates
# 6. Stop when done
curl -X DELETE http://127.0.0.1:8070/sandbox/abc123
```

### **7. Advanced Features**

#### **Add Files Later**
```bash
curl -X POST http://127.0.0.1:8070/sandbox/{id}/files \
  -H "Content-Type: application/json" \
  -d '[{
    "path": "src/components/Header.tsx",
    "content": "import React from \"react\";\n\nconst Header = () => <header>My Header</header>;\n\nexport default Header;"
  }]'
```

#### **Custom Scripts**
```json
{
  "entry_point": "bun run build && bun run preview",
  "files": [{
    "path": "package.json",
    "content": "{\n  \"scripts\": {\n    \"build\": \"bun build src/index.tsx --outdir=dist\",\n    \"preview\": \"bun run dist/index.js\"\n  }\n}"
  }]
}
```

### **8. Benefits vs Traditional Development**

| Traditional | Sandbox |
|-------------|---------|
| Install locally | ✅ No local installation |
| Version conflicts | ✅ Isolated environment |
| System dependencies | ✅ Container handles everything |
| Manual setup | ✅ Automatic setup |
| Limited isolation | ✅ Complete isolation |
| Hard to share | ✅ Easy to share/reproduce |

### **9. Perfect for Your Use Case**

This setup is **perfect** for:
- ✅ **Multi-file TypeScript projects** 
- ✅ **React applications**
- ✅ **Development servers that need to stay running**
- ✅ **Hot reload development**
- ✅ **Automatic dependency management**
- ✅ **Isolated development environments**

### **10. Ready to Use!**

Your TypeScript project with App.tsx, Page.tsx, styles.css, and more can be:
1. **Transferred** in a single API call
2. **Automatically set up** with `bun install`
3. **Kept running** with persistent containers
4. **Accessed** via development server
5. **Stopped** when you're done

The sandbox gives you a **complete development environment** that's isolated, reproducible, and ready to use! 🚀