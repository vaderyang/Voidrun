# 🔄 Reverse Proxy Solution for Sandbox Web Service Access

## **Problem Statement**
Clients need to access web services (dev servers) running inside sandbox containers. Currently:
- Containers bind to `127.0.0.1:3000` (hardcoded)
- Multiple sandboxes create port conflicts
- No discovery mechanism for clients

## **Implemented Solution: Central Reverse Proxy**

### **1. Architecture Overview**
```
Client Request
     ↓
Sandbox Service (Port 8070)
     ↓
Reverse Proxy Router
     ↓
Individual Container Ports (8070, 8081, 8082...)
```

### **2. URL Structure**
```
http://127.0.0.1:8070/sandbox/{sandbox_id}/        # Proxy to container
http://127.0.0.1:8070/sandbox/{sandbox_id}/api/... # Proxy API calls
http://127.0.0.1:8070/sandbox/{sandbox_id}/assets/... # Proxy static assets
```

### **3. Implementation Details**

#### **A. Port Allocation System**
- **Dynamic Port Assignment**: Each sandbox gets a unique port (8070, 8081, 8082...)
- **Port Tracking**: `PortAllocator` manages port assignments per sandbox
- **Automatic Cleanup**: Ports are released when sandboxes are deleted

#### **B. Reverse Proxy Router**
- **Path-based Routing**: `/sandbox/{sandbox_id}/*` routes to corresponding container
- **Header Forwarding**: Preserves client headers and cookies
- **Error Handling**: Returns 404 for non-existent sandboxes, 502 for container errors

#### **C. Container Configuration**
- **Dynamic Port Binding**: Containers bind to allocated host ports
- **Network Access**: Persistent containers get bridge network access
- **Port Forwarding**: Container port 3000 → Host port (allocated)

### **4. Client Usage Examples**

#### **Create a Sandbox**
```bash
curl -X POST http://127.0.0.1:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{"runtime": "bun", "mode": "persistent", "dev_server": true, ...}'
```

#### **Access the Web Service**
```bash
# Direct proxy access
curl http://127.0.0.1:8070/sandbox/{sandbox_id}/

# API calls through proxy
curl http://127.0.0.1:8070/sandbox/{sandbox_id}/api/users

# Static assets through proxy
curl http://127.0.0.1:8070/sandbox/{sandbox_id}/assets/style.css
```

### **5. Key Benefits**

✅ **No Port Conflicts**: Each sandbox gets a unique port
✅ **Client-Friendly**: Single entry point through sandbox service
✅ **Automatic Discovery**: Clients don't need to know individual ports
✅ **Secure**: Only the sandbox service port needs to be exposed
✅ **Scalable**: Supports multiple concurrent sandboxes

### **6. Configuration Options**

```rust
// In main.rs
let proxy_state = ProxyState::new(8070); // Start port allocation from 8070

// Router setup
let api_router = create_router(app_state.clone());
let proxy_router = create_proxy_router(proxy_state);
let app = api_router.merge(proxy_router);
```

### **7. Enhanced Sandbox Response**

When creating a sandbox, the response includes:
```json
{
  "id": "abc123",
  "status": "running",
  "runtime": "bun",
  "dev_server_url": "http://127.0.0.1:8070/sandbox/abc123/",
  "allocated_port": 8070,
  "proxy_enabled": true
}
```

### **8. Error Handling**

- **404 Not Found**: Sandbox doesn't exist or no port allocated
- **502 Bad Gateway**: Container is down or unreachable
- **500 Internal Server Error**: Proxy configuration issues

### **9. Production Considerations**

#### **Security**
- Add authentication/authorization before proxy
- Implement rate limiting per sandbox
- Add request logging and monitoring

#### **Performance**
- Connection pooling for upstream requests
- Caching for static assets
- Load balancing for multiple container instances

#### **Scalability**
- Support for multiple host machines
- Integration with container orchestration
- Dynamic scaling based on usage

### **10. Alternative Solutions Considered**

#### **A. Direct Port Exposure**
- **Pros**: Simpler implementation
- **Cons**: Port conflicts, client complexity, security issues

#### **B. Subdomain Routing**
- **Pros**: Clean URLs (sandbox-abc123.localhost)
- **Cons**: DNS configuration, certificate management

#### **C. WebSocket Proxy**
- **Pros**: Real-time updates
- **Cons**: More complex implementation

### **11. Current Status**

✅ **Port Allocation System**: Complete
✅ **Reverse Proxy Implementation**: Complete
✅ **Router Integration**: Complete
🔄 **Container Port Integration**: Partial (needs port allocation in Docker backend)
⏳ **Testing**: Pending

### **12. Next Steps**

1. **Complete Docker Backend Integration**
   - Pass allocated ports to container creation
   - Update sandbox responses with proxy URLs

2. **Testing**
   - Test proxy functionality with real TypeScript projects
   - Verify port allocation and cleanup

3. **Documentation**
   - Update API documentation
   - Add client usage examples

## **Conclusion**

The reverse proxy solution provides a clean, scalable approach to accessing sandbox web services. Clients interact with a single endpoint while the system handles port allocation, routing, and container management transparently.

This approach eliminates port conflicts, simplifies client code, and provides a foundation for advanced features like authentication, rate limiting, and monitoring.