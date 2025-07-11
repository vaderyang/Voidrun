// Simple test script to verify the FaaS service is working
console.log("Testing FaaS Service...");

// Test 1: Check if service is running
fetch('http://localhost:8070/health')
  .then(response => response.json())
  .then(data => {
    console.log("✓ Service Health Check:", data);
    
    // Test 2: List deployments
    return fetch('http://localhost:8070/faas/deployments');
  })
  .then(response => response.json())
  .then(data => {
    console.log("✓ List Deployments:", data);
    
    // Test 3: Try simple deployment
    const deployment = {
      runtime: "bun",
      code: "console.log('Hello from FaaS!');"
    };
    
    console.log("Attempting deployment...");
    return fetch('http://localhost:8070/faas/deploy', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(deployment)
    });
  })
  .then(response => {
    console.log("Deploy response status:", response.status);
    return response.text();
  })
  .then(text => {
    console.log("Deploy response:", text);
    if (text.trim()) {
      try {
        const data = JSON.parse(text);
        console.log("✓ Deployment successful:", data);
      } catch (e) {
        console.log("✗ Deployment failed - invalid JSON:", text);
      }
    } else {
      console.log("✗ Deployment failed - empty response");
    }
  })
  .catch(error => {
    console.error("✗ Error:", error);
  }); 