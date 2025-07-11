#!/bin/bash

BASE_URL="http://127.0.0.1:8070"
SANDBOX_ID_FILE="sandbox_id.txt"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

show_help() {
    echo -e "${BLUE}🚀 TypeScript Sandbox Manager${NC}"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  create    - Create new persistent TypeScript sandbox"
    echo "  status    - Check sandbox status"
    echo "  logs      - Show execution logs"
    echo "  list      - List all sandboxes"
    echo "  stop      - Stop and delete sandbox"
    echo "  help      - Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 create    # Create new project"
    echo "  $0 status    # Check if running"
    echo "  $0 stop      # Stop when done"
}

create_sandbox() {
    echo -e "${BLUE}🚀 Creating TypeScript sandbox...${NC}"
    
    RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox" \
        -H "Content-Type: application/json" \
        -d @my_typescript_project.json)
    
    if [ $? -eq 0 ]; then
        SANDBOX_ID=$(echo "$RESPONSE" | jq -r '.id')
        echo "$SANDBOX_ID" > "$SANDBOX_ID_FILE"
        echo -e "${GREEN}✅ Sandbox created: $SANDBOX_ID${NC}"
        echo -e "${YELLOW}📦 Installing dependencies and starting dev server...${NC}"
        echo -e "${BLUE}💡 Use '$0 status' to check progress${NC}"
    else
        echo -e "${RED}❌ Failed to create sandbox${NC}"
        exit 1
    fi
}

check_status() {
    if [ ! -f "$SANDBOX_ID_FILE" ]; then
        echo -e "${RED}❌ No sandbox ID found. Create one first with '$0 create'${NC}"
        exit 1
    fi
    
    SANDBOX_ID=$(cat "$SANDBOX_ID_FILE")
    echo -e "${BLUE}📊 Checking sandbox status...${NC}"
    
    RESPONSE=$(curl -s "$BASE_URL/sandbox/$SANDBOX_ID")
    if [ $? -eq 0 ]; then
        echo "$RESPONSE" | jq
        
        STATUS=$(echo "$RESPONSE" | jq -r '.status')
        case $STATUS in
            "Created")
                echo -e "${YELLOW}⏳ Sandbox is initializing...${NC}"
                ;;
            "Installing")
                echo -e "${YELLOW}📦 Installing dependencies...${NC}"
                ;;
            "Running")
                echo -e "${GREEN}🚀 Development server is running!${NC}"
                echo -e "${BLUE}🌐 Access your app at: http://localhost:3000${NC}"
                ;;
            "DevServer")
                echo -e "${GREEN}🎉 Development server is active!${NC}"
                echo -e "${BLUE}🌐 Access your app at: http://localhost:3000${NC}"
                ;;
            "Failed")
                echo -e "${RED}❌ Sandbox failed to start${NC}"
                ;;
            *)
                echo -e "${YELLOW}📋 Status: $STATUS${NC}"
                ;;
        esac
    else
        echo -e "${RED}❌ Failed to get sandbox status${NC}"
        exit 1
    fi
}

show_logs() {
    if [ ! -f "$SANDBOX_ID_FILE" ]; then
        echo -e "${RED}❌ No sandbox ID found. Create one first with '$0 create'${NC}"
        exit 1
    fi
    
    SANDBOX_ID=$(cat "$SANDBOX_ID_FILE")
    echo -e "${BLUE}📜 Execution logs:${NC}"
    
    RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox/$SANDBOX_ID/execute")
    if [ $? -eq 0 ]; then
        echo "$RESPONSE" | jq
    else
        echo -e "${RED}❌ Failed to get logs${NC}"
        exit 1
    fi
}

list_sandboxes() {
    echo -e "${BLUE}📋 All sandboxes:${NC}"
    curl -s "$BASE_URL/sandbox" | jq
}

stop_sandbox() {
    if [ ! -f "$SANDBOX_ID_FILE" ]; then
        echo -e "${RED}❌ No sandbox ID found${NC}"
        exit 1
    fi
    
    SANDBOX_ID=$(cat "$SANDBOX_ID_FILE")
    echo -e "${YELLOW}🛑 Stopping sandbox: $SANDBOX_ID${NC}"
    
    RESPONSE=$(curl -s -X DELETE "$BASE_URL/sandbox/$SANDBOX_ID")
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Sandbox stopped and cleaned up${NC}"
        rm -f "$SANDBOX_ID_FILE"
    else
        echo -e "${RED}❌ Failed to stop sandbox${NC}"
        exit 1
    fi
}

# Main command handling
case "$1" in
    create)
        create_sandbox
        ;;
    status)
        check_status
        ;;
    logs)
        show_logs
        ;;
    list)
        list_sandboxes
        ;;
    stop)
        stop_sandbox
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac