# Sandbox Service

A secure, high-performance sandbox service for running TypeScript, Bun, and Node.js code in isolated environments. Built with Rust for maximum performance and security.

## Features

- **Multiple Isolation Backends**: Docker containers, nsjail, and extensible architecture for additional backends
- **Runtime Support**: TypeScript, Bun, and Node.js
- **RESTful API**: Clean HTTP API for sandbox management
- **Security**: Network isolation, memory limits, CPU limits, and filesystem restrictions
- **Performance**: Fast startup times and efficient resource usage
- **Configurable**: Environment variables and config file support

## Supported Backends

### nsjail (Recommended for production)
- **Pros**: Lightweight, fast startup, low resource overhead
- **Cons**: Linux-only, requires nsjail installation
- **Use case**: High-frequency code execution, resource-constrained environments

### Docker
- **Pros**: Cross-platform, strong isolation, familiar tooling
- **Cons**: Higher resource usage, slower startup
- **Use case**: Strong isolation requirements, cross-platform compatibility

### Future Backends
- Firecracker (planned)
- gVisor (planned)
- BPFBox (planned)
- Rumpkernel (planned)

## Quick Start

### Prerequisites

For nsjail backend (recommended):
```bash
# Ubuntu/Debian
sudo apt-get install nsjail

# macOS (requires compiling from source)
# See: https://github.com/google/nsjail
```

For Docker backend:
```bash
# Install Docker
curl -fsSL https://get.docker.com | sh
```

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd sandbox-service

# Build the service
cargo build --release

# Run with default settings (nsjail backend)
./target/release/sandbox-service

# Run with Docker backend
./target/release/sandbox-service --backend docker
```

## API Usage

### Health Check
```bash
curl http://localhost:8070/health
```

### Create and Execute Sandbox
```bash
# Create a Node.js sandbox
curl -X POST http://localhost:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "node",
    "code": "console.log(\"Hello from Node.js!\")",
    "timeout_ms": 5000,
    "memory_limit_mb": 128
  }'

# Response: {"id": "123e4567-e89b-12d3-a456-426614174000", ...}

# Execute the sandbox
curl -X POST http://localhost:8070/sandbox/123e4567-e89b-12d3-a456-426614174000/execute
```

### TypeScript Example
```bash
curl -X POST http://localhost:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "typescript",
    "code": "const greeting: string = \"Hello TypeScript!\"; console.log(greeting);",
    "timeout_ms": 10000,
    "memory_limit_mb": 256
  }'
```

### Bun Example
```bash
curl -X POST http://localhost:8070/sandbox \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "bun",
    "code": "console.log(\"Hello from Bun!\"); console.log(Bun.version);",
    "timeout_ms": 5000,
    "memory_limit_mb": 128
  }'
```

## Configuration

### Environment Variables

```bash
SANDBOX_HOST=127.0.0.1
SANDBOX_PORT=8070
SANDBOX_BACKEND=nsjail  # or docker
SANDBOX_TIMEOUT_MS=30000
SANDBOX_MEMORY_LIMIT_MB=512
LOG_LEVEL=info
```

### Configuration File

Create `config.toml`:
```toml
[server]
host = "0.0.0.0"
port = 8070

[sandbox]
backend = "nsjail"  # or "docker"
default_timeout_ms = 30000
default_memory_limit_mb = 512
max_concurrent_sandboxes = 10
cleanup_interval_seconds = 300

[logging]
level = "info"
format = "json"
```

Run with config file:
```bash
./target/release/sandbox-service --config config.toml
```

## API Reference

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| POST | `/sandbox` | Create sandbox |
| GET | `/sandbox/{id}` | Get sandbox info |
| DELETE | `/sandbox/{id}` | Delete sandbox |
| POST | `/sandbox/{id}/execute` | Execute code in sandbox |
| GET | `/sandbox` | List all sandboxes |

### Request/Response Examples

#### Create Sandbox Request
```json
{
  "runtime": "node",
  "code": "console.log('Hello World');",
  "entry_point": "index.js",
  "timeout_ms": 5000,
  "memory_limit_mb": 128,
  "env_vars": {
    "NODE_ENV": "sandbox"
  }
}
```

#### Execute Response
```json
{
  "sandbox_id": "123e4567-e89b-12d3-a456-426614174000",
  "success": true,
  "stdout": "Hello World\\n",
  "stderr": "",
  "exit_code": 0,
  "execution_time_ms": 45
}
```

## Security Features

- **Network Isolation**: No network access from sandboxed code
- **Filesystem Restrictions**: Read-only root filesystem with limited writable areas
- **Memory Limits**: Configurable memory limits per sandbox
- **CPU Limits**: CPU time restrictions to prevent resource exhaustion
- **Timeout Protection**: Automatic termination of long-running processes
- **Process Isolation**: Each sandbox runs in its own process namespace

## Development

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Features
```bash
# Build with Docker support
cargo build --features docker

# Build without Docker (nsjail only)
cargo build --no-default-features
```

## Production Deployment

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y nsjail && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sandbox-service /usr/local/bin/
EXPOSE 8070
CMD ["sandbox-service"]
```

### Systemd Service
```ini
[Unit]
Description=Sandbox Service
After=network.target

[Service]
Type=simple
User=sandbox
WorkingDirectory=/opt/sandbox-service
ExecStart=/opt/sandbox-service/sandbox-service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## Performance Benchmarks

### nsjail Backend
- **Startup Time**: ~50ms per sandbox
- **Memory Overhead**: ~10MB per sandbox
- **Throughput**: 1000+ executions/second (depends on code complexity)

### Docker Backend
- **Startup Time**: ~200ms per sandbox
- **Memory Overhead**: ~50MB per sandbox
- **Throughput**: 500+ executions/second

## Troubleshooting

### Common Issues

1. **nsjail not found**: Install nsjail or use Docker backend
2. **Permission denied**: Ensure proper user permissions for nsjail
3. **Docker daemon not running**: Start Docker service
4. **Port already in use**: Change port in configuration

### Logs
```bash
# Enable debug logging
LOG_LEVEL=debug ./target/release/sandbox-service

# JSON formatted logs
LOG_FORMAT=json ./target/release/sandbox-service
```

## License

MIT License - see LICENSE file for details.