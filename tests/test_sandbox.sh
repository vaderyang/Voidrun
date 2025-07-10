#!/bin/bash

BASE_URL="http://127.0.0.1:8070"

echo "=== Sandbox Service Test Script ==="
echo "Testing sandbox service at $BASE_URL"

# Test 1: Health check
echo -e "\n1. Testing health endpoint..."
curl -s "$BASE_URL/health" | jq .
if [ $? -eq 0 ]; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed"
    exit 1
fi

# Test 2: Create Node.js sandbox
echo -e "\n2. Creating Node.js sandbox..."
CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox" \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "node",
    "code": "console.log(\"Hello World!\"); console.log(\"Test successful!\");",
    "timeout_ms": 5000,
    "memory_limit_mb": 128
  }')

echo "Create response: $CREATE_RESPONSE"
SANDBOX_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')

if [ "$SANDBOX_ID" == "null" ] || [ -z "$SANDBOX_ID" ]; then
    echo "❌ Failed to create sandbox"
    exit 1
fi

echo "✅ Sandbox created with ID: $SANDBOX_ID"

# Test 3: Get sandbox info
echo -e "\n3. Getting sandbox info..."
INFO_RESPONSE=$(curl -s "$BASE_URL/sandbox/$SANDBOX_ID")
echo "Info response: $INFO_RESPONSE"

# Test 4: Execute sandbox
echo -e "\n4. Executing sandbox..."
EXEC_RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox/$SANDBOX_ID/execute")
echo "Execute response: $EXEC_RESPONSE"

# Test 5: List all sandboxes
echo -e "\n5. Listing all sandboxes..."
LIST_RESPONSE=$(curl -s "$BASE_URL/sandbox")
echo "List response: $LIST_RESPONSE"

# Test 6: Create TypeScript sandbox
echo -e "\n6. Creating TypeScript sandbox..."
TS_CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox" \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "typescript",
    "code": "interface User { name: string; } const user: User = { name: \"TypeScript\" }; console.log(`Hello, ${user.name}!`);",
    "timeout_ms": 10000,
    "memory_limit_mb": 256
  }')

echo "TypeScript create response: $TS_CREATE_RESPONSE"
TS_SANDBOX_ID=$(echo "$TS_CREATE_RESPONSE" | jq -r '.id')

if [ "$TS_SANDBOX_ID" != "null" ] && [ -n "$TS_SANDBOX_ID" ]; then
    echo "✅ TypeScript sandbox created with ID: $TS_SANDBOX_ID"
    
    # Execute TypeScript sandbox
    echo -e "\n7. Executing TypeScript sandbox..."
    TS_EXEC_RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox/$TS_SANDBOX_ID/execute")
    echo "TypeScript execute response: $TS_EXEC_RESPONSE"
else
    echo "❌ Failed to create TypeScript sandbox"
fi

# Test 7: Test error handling
echo -e "\n8. Testing error handling..."
ERROR_CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox" \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "node",
    "code": "throw new Error(\"Test error\");",
    "timeout_ms": 5000,
    "memory_limit_mb": 128
  }')

echo "Error test create response: $ERROR_CREATE_RESPONSE"
ERROR_SANDBOX_ID=$(echo "$ERROR_CREATE_RESPONSE" | jq -r '.id')

if [ "$ERROR_SANDBOX_ID" != "null" ] && [ -n "$ERROR_SANDBOX_ID" ]; then
    echo "✅ Error test sandbox created with ID: $ERROR_SANDBOX_ID"
    
    # Execute error sandbox
    echo -e "\n9. Executing error sandbox..."
    ERROR_EXEC_RESPONSE=$(curl -s -X POST "$BASE_URL/sandbox/$ERROR_SANDBOX_ID/execute")
    echo "Error execute response: $ERROR_EXEC_RESPONSE"
else
    echo "❌ Failed to create error test sandbox"
fi

echo -e "\n=== Test Complete ==="