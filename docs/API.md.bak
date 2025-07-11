# Sandbox Service API Documentation

The Sandbox Service provides a secure, isolated environment for executing code in various runtimes including Node.js, Bun, and TypeScript. This document covers both the main API endpoints and the Admin API.

## Base URLs

- **Main API**: `http://localhost:8070`
- **Admin API**: `http://localhost:8070/admin/api`
- **Admin UI**: `http://localhost:8070/admin`

## Authentication

Currently, the API does not require authentication. All endpoints are publicly accessible.

## Error Handling

The API uses standard HTTP status codes:

- `200` - Success
- `400` - Bad Request (invalid parameters)
- `404` - Not Found (sandbox doesn't exist)
- `500` - Internal Server Error

Error responses include a JSON object with error details:

```json
{
  "error": "Error message description"
}
```

---

## Main API Endpoints

### Health Check

Check if the service is running.

**GET** `/health`

#### Response
```json
{
  "status": "ok",
  "service": "sandbox-service",
  "version": "0.1.0"
}
```

#### Example
```bash
curl http://localhost:8070/health
```

---

### Create Sandbox

Create a new sandbox environment.

**POST** `/sandbox`

#### Request Body
```json
{
  "runtime": "node|bun|typescript",
  "code": "string",
  "entry_point": "string (optional)",
  "timeout_ms": "number (optional, default: 30000)",
  "memory_limit_mb": "number (optional, default: 256)",
  "env_vars": "object (optional)",
  "files": "array (optional)",
  "mode": "oneshot|persistent (optional, default: oneshot)",
  "install_deps": "boolean (optional, default: false)",
  "dev_server": "boolean (optional, default: false)"
}
```

#### Response
```json
{
  "id": "uuid",
  "status": "created",
  "runtime": "string",
  "created_at": "ISO 8601 timestamp",
  "timeout_ms": "number",
  "memory_limit_mb": "number"
}
```

#### Examples

**Basic Node.js Script**
```bash
curl -X POST http://localhost:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "node",
    "code": "console.log(\"Hello, World!\");"
  }'
```

**Persistent Node.js Server**
```bash
curl -X POST http://localhost:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "node",
    "code": "const http = require(\"http\"); const server = http.createServer((req, res) => { res.writeHead(200, {\"Content-Type\": \"text/plain\"}); res.end(\"Hello from sandbox!\"); }); server.listen(3000, () => console.log(\"Server running on port 3000\"));",
    "mode": "persistent",
    "dev_server": true,
    "timeout_ms": 300000
  }'
```

**TypeScript with Dependencies**
```bash
curl -X POST http://localhost:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "typescript",
    "code": "import { v4 as uuidv4 } from \"uuid\"; console.log(\"Random UUID:\", uuidv4());",
    "install_deps": true,
    "files": [
      {
        "path": "package.json",
        "content": "{\"dependencies\": {\"uuid\": \"^9.0.0\", \"@types/uuid\": \"^9.0.0\"}}"
      }
    ]
  }'
```

**Bun with Environment Variables**
```bash
curl -X POST http://localhost:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "bun",
    "code": "console.log(\"Environment:\", process.env.NODE_ENV); console.log(\"API Key:\", process.env.API_KEY);",
    "env_vars": {
      "NODE_ENV": "development",
      "API_KEY": "test-key-123"
    }
  }'
```

---

### Get Sandbox Info

Retrieve information about a specific sandbox.

**GET** `/sandbox/{id}`

#### Response
```json
{
  "id": "uuid",
  "status": "created|running|completed|failed",
  "runtime": "string",
  "created_at": "ISO 8601 timestamp",
  "timeout_ms": "number",
  "memory_limit_mb": "number"
}
```

#### Example
```bash
curl http://localhost:8070/sandbox/fab81d7c-f665-432b-85c4-f9d380019709
```

---

### Execute Code

Execute code in an existing sandbox.

**POST** `/sandbox/{id}/execute`

#### Response
```json
{
  "sandbox_id": "uuid",
  "success": "boolean",
  "stdout": "string",
  "stderr": "string",
  "exit_code": "number",
  "execution_time_ms": "number"
}
```

#### Example
```bash
curl -X POST http://localhost:8070/sandbox/fab81d7c-f665-432b-85c4-f9d380019709/execute
```

---

### Upload Files

Upload additional files to an existing sandbox.

**POST** `/sandbox/{id}/files`

#### Request Body
```json
[
  {
    "path": "string",
    "content": "string",
    "is_executable": "boolean (optional)"
  }
]
```

#### Response
```json
{
  "message": "Files uploaded successfully",
  "sandbox_id": "uuid"
}
```

#### Example
```bash
curl -X POST http://localhost:8070/sandbox/fab81d7c-f665-432b-85c4-f9d380019709/files \
  -H "Content-Type: application/json" \
  -d '[
    {
      "path": "utils.js",
      "content": "exports.formatDate = (date) => date.toISOString();"
    },
    {
      "path": "run.sh",
      "content": "#!/bin/bash\nnode index.js",
      "is_executable": true
    }
  ]'
```

---

### List Sandboxes

List all sandboxes.

**GET** `/sandboxes`

#### Response
```json
[
  {
    "id": "uuid",
    "status": "string",
    "runtime": "string",
    "created_at": "ISO 8601 timestamp",
    "timeout_ms": "number",
    "memory_limit_mb": "number"
  }
]
```

#### Example
```bash
curl http://localhost:8070/sandboxes
```

---

### Delete Sandbox

Delete a sandbox and clean up its resources.

**DELETE** `/sandbox/{id}`

#### Response
- Status: `204 No Content` on success
- Status: `404 Not Found` if sandbox doesn't exist

#### Example
```bash
curl -X DELETE http://localhost:8070/sandbox/fab81d7c-f665-432b-85c4-f9d380019709
```

---

## Admin API Endpoints

The Admin API provides enhanced monitoring and management capabilities.

### System Status

Get overall system status and resource usage.

**GET** `/admin/api/status`

#### Response
```json
{
  "uptime": "number (seconds)",
  "active_sandboxes": "number",
  "total_sandboxes_created": "number",
  "backend_type": "Docker|Nsjail",
  "version": "string",
  "memory_usage": {
    "used": "number (MB)",
    "total": "number (MB)",
    "percentage": "number"
  },
  "cpu_usage": {
    "used": "number (percentage)",
    "total": 100.0,
    "percentage": "number"
  }
}
```

#### Example
```bash
curl http://localhost:8070/admin/api/status
```

---

### List Sandboxes (Admin)

Get detailed information about all sandboxes.

**GET** `/admin/api/sandboxes`

#### Response
```json
[
  {
    "id": "uuid",
    "status": "string",
    "runtime": "string",
    "created_at": "ISO 8601 timestamp",
    "uptime": "number (seconds)",
    "memory_mb": "number",
    "cpu_percentage": "number",
    "dev_server_url": "string (optional)",
    "allocated_port": "number (optional)",
    "is_persistent": "boolean",
    "container_id": "string (optional)"
  }
]
```

#### Example
```bash
curl http://localhost:8070/admin/api/sandboxes
```

---

### Get Sandbox Details (Admin)

Get detailed information about a specific sandbox.

**GET** `/admin/api/sandboxes/{id}`

#### Response
```json
{
  "id": "uuid",
  "status": "string",
  "runtime": "string",
  "created_at": "ISO 8601 timestamp",
  "uptime": "number (seconds)",
  "memory_mb": "number",
  "cpu_percentage": "number",
  "dev_server_url": "string (optional)",
  "allocated_port": "number (optional)",
  "is_persistent": "boolean",
  "container_id": "string (optional)"
}
```

#### Example
```bash
curl http://localhost:8070/admin/api/sandboxes/fab81d7c-f665-432b-85c4-f9d380019709
```

---

### Get Sandbox Logs

Retrieve logs from a specific sandbox.

**GET** `/admin/api/sandboxes/{id}/logs?lines={number}&follow={boolean}`

#### Query Parameters
- `lines` (optional): Number of log lines to retrieve (default: 100)
- `follow` (optional): Whether to follow logs in real-time (default: false)

#### Response
```json
[
  {
    "timestamp": "ISO 8601 timestamp",
    "level": "INFO|WARN|ERROR|DEBUG",
    "message": "string",
    "sandbox_id": "uuid"
  }
]
```

#### Example
```bash
curl "http://localhost:8070/admin/api/sandboxes/fab81d7c-f665-432b-85c4-f9d380019709/logs?lines=50"
```

---

### Force Stop Sandbox

Force stop a sandbox (useful for deadlocked sandboxes).

**POST** `/admin/api/sandboxes/{id}/force-stop`

#### Response
```json
{
  "success": "boolean",
  "message": "string"
}
```

#### Example
```bash
curl -X POST http://localhost:8070/admin/api/sandboxes/fab81d7c-f665-432b-85c4-f9d380019709/force-stop
```

---

### Get Sandbox Resources

Get detailed resource usage for a specific sandbox.

**GET** `/admin/api/sandboxes/{id}/resources`

#### Response
```json
{
  "memory": {
    "used": "number (MB)",
    "limit": "number (MB)",
    "percentage": "number"
  },
  "cpu": {
    "percentage": "number",
    "cores": "number"
  },
  "disk": {
    "read_bytes": "number",
    "write_bytes": "number",
    "used": "number (MB)",
    "limit": "number (MB)",
    "percentage": "number"
  },
  "network": {
    "bytes_in": "number",
    "bytes_out": "number"
  }
}
```

#### Example
```bash
curl http://localhost:8070/admin/api/sandboxes/fab81d7c-f665-432b-85c4-f9d380019709/resources
```

---

### Get System Logs

Retrieve system-wide logs.

**GET** `/admin/api/logs?lines={number}`

#### Query Parameters
- `lines` (optional): Number of log lines to retrieve (default: 100)

#### Response
```json
[
  {
    "timestamp": "ISO 8601 timestamp",
    "level": "INFO|WARN|ERROR|DEBUG",
    "message": "string",
    "sandbox_id": "uuid (optional)"
  }
]
```

#### Example
```bash
curl "http://localhost:8070/admin/api/logs?lines=100"
```

---

### Get API Documentation

Get programmatic API documentation.

**GET** `/admin/api/docs`

#### Response
```json
[
  {
    "method": "string",
    "path": "string",
    "description": "string",
    "parameters": [
      {
        "name": "string",
        "param_type": "string",
        "required": "boolean",
        "description": "string"
      }
    ],
    "example_request": "string (optional)",
    "example_response": "string (optional)"
  }
]
```

#### Example
```bash
curl http://localhost:8070/admin/api/docs
```

---

### Test API Endpoint

Test API endpoints through the admin interface.

**POST** `/admin/api/test`

#### Request Body
```json
{
  "method": "GET|POST|PUT|DELETE|PATCH",
  "path": "string",
  "headers": "object (optional)",
  "body": "string (optional)"
}
```

#### Response
```json
{
  "status": "number",
  "headers": "object",
  "body": "string",
  "duration_ms": "number"
}
```

#### Example
```bash
curl -X POST http://localhost:8070/admin/api/test \
  -H "Content-Type: application/json" \
  -d '{
    "method": "GET",
    "path": "/health"
  }'
```

---

## FaaS/Serverless API Endpoints

The FaaS API provides serverless function deployment with automatic lifecycle management, eliminating the need to manually handle sandbox creation and deletion.

### Deploy Function

Deploy a new serverless function with automatic container management.

**POST** `/faas/deploy`

#### Request Body
```json
{
  "runtime": "bun|node|typescript",
  "code": "string",
  "files": "array (optional)",
  "env_vars": "object (optional)",
  "memory_limit_mb": "number (optional, default: 256)",
  "entry_point": "string (optional)",
  "auto_scale": "object (optional)",
  "dev_server": "boolean (optional, default: true)"
}
```

#### Response
```json
{
  "deployment_id": "uuid",
  "url": "string",
  "sandbox_id": "uuid",
  "status": "Running",
  "created_at": "ISO 8601 timestamp",
  "runtime": "string",
  "memory_mb": "number"
}
```

#### Example
```bash
curl -X POST http://localhost:8070/faas/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "bun",
    "code": "console.log(\"Hello FaaS!\");",
    "entry_point": "bun dev",
    "files": [
      {
        "path": "package.json",
        "content": "{\"name\": \"my-function\", \"scripts\": {\"dev\": \"bun run --hot index.ts\"}}"
      },
      {
        "path": "index.ts",
        "content": "import express from \"express\"; const app = express(); app.get(\"/\", (req, res) => res.json({message: \"Hello from FaaS!\"})); app.listen(3000);"
      }
    ],
    "auto_scale": {
      "min_instances": 0,
      "max_instances": 5,
      "scale_down_after_minutes": 10
    }
  }'
```

---

### List Deployments

List all active FaaS deployments.

**GET** `/faas/deployments`

#### Response
```json
[
  {
    "deployment_id": "uuid",
    "url": "string",
    "sandbox_id": "uuid",
    "status": "Running",
    "created_at": "ISO 8601 timestamp",
    "runtime": "string",
    "memory_mb": "number"
  }
]
```

#### Example
```bash
curl http://localhost:8070/faas/deployments
```

---

### Get Deployment Info

Get information about a specific deployment.

**GET** `/faas/deployments/{deployment_id}`

#### Response
```json
{
  "deployment_id": "uuid",
  "url": "string",
  "sandbox_id": "uuid",
  "status": "Running",
  "created_at": "ISO 8601 timestamp",
  "runtime": "string",
  "memory_mb": "number"
}
```

#### Example
```bash
curl http://localhost:8070/faas/deployments/4a5fded3-e704-40fa-84a5-fda2bc7ea548
```

---

### Update Files in Deployment

Update files in a running deployment with automatic dev server restart.

**PUT** `/faas/deployments/{deployment_id}/files`

#### Request Body
```json
{
  "files": [
    {
      "path": "string",
      "content": "string",
      "executable": "boolean (optional)"
    }
  ],
  "restart_dev_server": "boolean (optional, default: true)"
}
```

#### Response
- Status: `200 OK` on success
- Status: `404 Not Found` if deployment doesn't exist

#### Example
```bash
curl -X PUT http://localhost:8070/faas/deployments/4a5fded3-e704-40fa-84a5-fda2bc7ea548/files \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "path": "index.ts",
        "content": "import express from \"express\"; const app = express(); app.get(\"/\", (req, res) => res.json({message: \"Updated via API!\", timestamp: new Date().toISOString()})); app.listen(3000);"
      }
    ],
    "restart_dev_server": true
  }'
```

---

### Undeploy Function

Remove a deployment and clean up all resources.

**DELETE** `/faas/deployments/{deployment_id}`

#### Response
- Status: `204 No Content` on success
- Status: `404 Not Found` if deployment doesn't exist

#### Example
```bash
curl -X DELETE http://localhost:8070/faas/deployments/4a5fded3-e704-40fa-84a5-fda2bc7ea548
```

---

## Proxy Endpoints

The service provides proxy access to both sandboxes and FaaS deployments:

### Sandbox Proxy
**GET/POST/PUT/DELETE** `/proxy/{sandbox_id}/*`

Forwards requests to the sandbox's internal dev server running on port 3000.

#### Example
```bash
curl http://localhost:8070/proxy/fab81d7c-f665-432b-85c4-f9d380019709/
```

### FaaS Proxy
**GET/POST/PUT/DELETE** `/faas/{deployment_id}/*`

Forwards requests to the FaaS deployment's web service.

#### Example
```bash
# Access the root endpoint of your FaaS deployment
curl http://localhost:8070/faas/4a5fded3-e704-40fa-84a5-fda2bc7ea548/

# Access specific API endpoints
curl http://localhost:8070/faas/4a5fded3-e704-40fa-84a5-fda2bc7ea548/api/status
curl http://localhost:8070/faas/4a5fded3-e704-40fa-84a5-fda2bc7ea548/health
```

---

## Runtime Support

### Node.js (`runtime: "node"`)
- Supports CommonJS modules
- Built-in Node.js modules available
- NPM packages can be installed with `install_deps: true`

### Bun (`runtime: "bun"`)
- TypeScript support out of the box
- Fast package installation
- Modern JavaScript features

### TypeScript (`runtime: "typescript"`)
- Full TypeScript support
- Automatic compilation
- Type checking enabled

---

## Sandbox Modes

### OneShot Mode (default)
- Executes code once and exits
- Suitable for scripts and batch processing
- Container is cleaned up after execution

### Persistent Mode
- Keeps the sandbox running
- Suitable for servers and long-running processes
- Supports dev server proxy
- Manual cleanup required

---

## File Management

Sandboxes support file uploads for:
- Configuration files (package.json, tsconfig.json)
- Source code files
- Scripts and utilities
- Assets and data files

Files can be marked as executable and will be given appropriate permissions.

---

## Resource Limits

### Default Limits
- **Memory**: 512MB per sandbox
- **Timeout**: 30 seconds for oneshot, configurable for persistent
- **CPU**: Shared, with 50% quota per container

### Customization
All limits can be customized per sandbox:
```json
{
  "memory_limit_mb": 1024,
  "timeout_ms": 60000
}
```

---

## Best Practices

1. **Use appropriate timeouts**: Set reasonable timeouts for long-running operations
2. **Clean up resources**: Delete sandboxes when no longer needed
3. **Monitor resource usage**: Use admin endpoints to monitor system health
4. **Handle errors gracefully**: Always check response status codes
5. **Use persistent mode judiciously**: Only use for actual long-running services
6. **Prefer FaaS for web services**: Use the FaaS API for deploying web applications and APIs
7. **Use file updates for development**: Leverage the file update API for iterative development
8. **Set appropriate auto-scale settings**: Configure reasonable scale-down timeouts for your use case

---

## Security Considerations

- Sandboxes are isolated using Docker containers
- Network access is limited based on sandbox mode
- File system access is restricted to the sandbox environment
- No access to host system resources
- Resource limits prevent resource exhaustion attacks

---

## WebSocket Support (Future)

Real-time log streaming and container monitoring via WebSocket connections will be available in future versions.

---

## Rate Limits

Currently, no rate limits are enforced, but they may be added in future versions for production deployments.
