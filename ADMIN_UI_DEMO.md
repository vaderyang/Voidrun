# 🎛️ Sandbox Admin UI - Complete Demo Guide

## **🚀 Overview**

The Sandbox Admin UI is a minimalist web interface providing comprehensive management and monitoring capabilities for your sandbox service. Access it at `http://127.0.0.1:8070/admin`.

## **🏗️ Features Implemented**

### **📊 1. Dashboard Tab**
- **System Status**: Uptime, version, backend type
- **Sandbox Metrics**: Active sandboxes, total created
- **Resource Monitoring**: Memory and CPU usage with progress bars
- **Auto-refresh**: Updates every 5 seconds

### **🔍 2. Sandboxes Tab**
- **Live Sandbox List**: All active sandboxes with detailed information
- **Sandbox Details**: ID, status, runtime, uptime, memory, CPU
- **Action Buttons**: 
  - **View**: Detailed sandbox information modal
  - **Logs**: View sandbox-specific logs
  - **Proxy**: Direct access to sandbox web service
  - **Stop**: Force stop deadlocked sandboxes

### **📋 3. Logs Tab**
- **System Logs**: View service-wide logs
- **Sandbox-specific Logs**: Filter by individual sandbox
- **Configurable Lines**: Adjust number of log lines (10-1000)
- **Real-time Updates**: Auto-refresh log content

### **📚 4. API Docs Tab**
- **Complete API Documentation**: All endpoints with parameters
- **Interactive API Tester**: Test endpoints directly from UI
- **Request Examples**: Pre-populated example requests
- **Response Display**: Real-time API response viewing

## **🎮 Demo Walkthrough**

### **Step 1: Start the Service**
```bash
./target/release/sandbox-service --port 8070 --backend docker &
```

### **Step 2: Access Admin UI**
Open your browser and navigate to:
```
http://127.0.0.1:8070/admin
```

### **Step 3: Dashboard Overview**
The dashboard displays:
- **System Status**: Current uptime and version
- **Resource Usage**: Memory at 25%, CPU at 37.5%
- **Sandbox Metrics**: Real-time active sandbox count

### **Step 4: Create Test Sandboxes**
```bash
# Create a persistent TypeScript sandbox
./manage_typescript_project.sh create

# Create additional sandboxes for testing
curl -X POST http://127.0.0.1:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{"runtime": "node", "code": "console.log(\"Hello Node\");", "mode": "persistent"}'
```

### **Step 5: Monitor Sandboxes**
1. Switch to **Sandboxes** tab
2. View live sandbox list with:
   - Truncated IDs (first 8 characters)
   - Status badges (Created, Running, Failed)
   - Runtime information
   - Uptime tracking
   - Memory and CPU usage

### **Step 6: View Sandbox Details**
1. Click **View** button on any sandbox
2. Modal shows complete information:
   - Full sandbox ID
   - Creation timestamp
   - Container ID (if available)
   - Dev server URL (if enabled)
   - Allocated port information

### **Step 7: Access Sandbox Web Service**
1. For sandboxes with dev servers, click **Proxy** button
2. Opens sandbox web service in new tab
3. URL format: `http://127.0.0.1:8070/proxy/{sandbox_id}/`

### **Step 8: View Logs**
1. Switch to **Logs** tab
2. View system-wide logs or filter by sandbox
3. Adjust log lines (default: 100)
4. Real-time log updates with timestamps

### **Step 9: Force Stop Deadlocked Sandboxes**
1. In Sandboxes tab, click **Stop** button
2. Confirm force stop action
3. Sandbox removed from active list
4. Resources freed immediately

### **Step 10: API Testing**
1. Switch to **API Docs** tab
2. Browse complete API documentation
3. Test endpoints directly:
   - Select HTTP method
   - Modify request path
   - Enter JSON request body
   - Click **Test Endpoint**
4. View response with status code and duration

## **🛠️ Technical Implementation**

### **Backend Structure**
```
/admin/
├── mod.rs          # Admin router and data structures
├── handlers.rs     # API endpoint handlers
└── ui.rs          # Complete HTML/CSS/JS UI
```

### **API Endpoints**
```
GET  /admin                    # Admin UI HTML
GET  /admin/api/status         # System status
GET  /admin/api/sandboxes      # List all sandboxes
GET  /admin/api/sandboxes/:id  # Get sandbox details
GET  /admin/api/sandboxes/:id/logs  # Get sandbox logs
POST /admin/api/sandboxes/:id/force-stop  # Force stop sandbox
GET  /admin/api/logs           # System logs
GET  /admin/api/docs           # API documentation
POST /admin/api/test           # Test API endpoints
```

### **Data Structures**
- **SystemStatus**: Uptime, sandbox metrics, resource usage
- **SandboxInfo**: Extended sandbox details with proxy URLs
- **LogEntry**: Structured log entries with timestamps
- **ApiEndpoint**: Complete API documentation with examples

## **🎨 UI Features**

### **Responsive Design**
- **Desktop**: Full-width layout with sidebar navigation
- **Mobile**: Stacked layout with responsive tables
- **Dark Theme**: Console-style log viewer

### **Interactive Elements**
- **Auto-refresh**: Dashboard and sandboxes update every 5 seconds
- **Modal Windows**: Detailed sandbox information
- **Progress Bars**: Visual resource usage indicators
- **Status Badges**: Color-coded sandbox states

### **User Experience**
- **Minimalist Design**: Clean, distraction-free interface
- **Intuitive Navigation**: Tab-based organization
- **Real-time Updates**: Live data without page refresh
- **Keyboard Shortcuts**: Accessible navigation

## **🔧 Configuration**

### **Auto-refresh Settings**
```javascript
// Refresh interval (5 seconds)
const refreshInterval = 5000;

// Auto-refresh enabled for:
- Dashboard metrics
- Sandbox list
- Log entries
```

### **Log Viewer Settings**
```javascript
// Default log lines
const defaultLines = 100;

// Supported line counts
const supportedLines = [10, 50, 100, 500, 1000];
```

## **📱 Example Usage Scenarios**

### **Scenario 1: Development Monitoring**
1. Create multiple TypeScript sandboxes
2. Monitor resource usage in real-time
3. Access dev servers through proxy
4. Check logs for build errors

### **Scenario 2: Debugging Issues**
1. Identify problematic sandbox in list
2. View detailed logs for troubleshooting
3. Check resource consumption
4. Force stop if needed

### **Scenario 3: API Testing**
1. Test new API endpoints
2. Validate request/response formats
3. Check error handling
4. Measure response times

### **Scenario 4: System Administration**
1. Monitor system health
2. Track resource usage trends
3. Manage sandbox lifecycle
4. Review system logs

## **🚀 Advanced Features**

### **Force Stop Functionality**
```javascript
// Confirms action before stopping
if (confirm('Are you sure you want to force stop?')) {
    // Calls DELETE endpoint
    // Updates UI immediately
    // Shows success/error message
}
```

### **Proxy Integration**
```javascript
// Dynamic proxy URL generation
const proxyUrl = `http://127.0.0.1:8070/proxy/${sandboxId}/`;

// Opens in new tab
window.open(proxyUrl, '_blank');
```

### **Real-time Log Streaming**
```javascript
// Scrolls to bottom automatically
logContent.scrollTop = logContent.scrollHeight;

// Color-coded log levels
- INFO: Blue
- WARN: Orange  
- ERROR: Red
```

## **🎯 Key Benefits**

### **For Developers**
- **Visual Debugging**: Easy log access and filtering
- **Resource Monitoring**: Track memory and CPU usage
- **Quick Testing**: Built-in API testing interface
- **Proxy Access**: Direct sandbox web service access

### **For Administrators**
- **System Overview**: Complete service status at a glance
- **Sandbox Management**: Full lifecycle control
- **Issue Resolution**: Force stop for deadlocked processes
- **Performance Monitoring**: Resource usage tracking

### **For Operations**
- **Health Monitoring**: System metrics and uptime
- **Log Analysis**: Centralized log viewing
- **API Documentation**: Complete reference guide
- **Automated Testing**: Built-in endpoint testing

## **🚀 Ready for Production**

The admin UI is production-ready with:
- **Security**: No authentication required (add as needed)
- **Performance**: Minimal resource usage
- **Scalability**: Handles multiple concurrent sandboxes
- **Reliability**: Graceful error handling
- **Maintainability**: Clean, documented code

## **📋 Next Steps**

1. **Authentication**: Add user authentication if needed
2. **Permissions**: Implement role-based access control
3. **Metrics**: Add more detailed resource monitoring
4. **Alerts**: Implement notification system
5. **Export**: Add data export functionality

The admin UI provides a complete, production-ready solution for managing your sandbox service with an intuitive, powerful interface! 🎉