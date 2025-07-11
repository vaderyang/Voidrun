# Sandbox Service

A secure, high-performance sandbox service for running TypeScript, Bun, and Node.js code in isolated environments. Built with Rust for maximum performance and security.

## Directory Structure

- `src/` — Rust source code
- `tests/` — All test files and test scripts (including advanced Bun/Node/TypeScript FaaS tests)
- `docs/` — All documentation and guides (see below)
- `examples/` — Rust code usage examples
- `README.md` — This file (quick start, API, and project overview)

## Features

- **Multiple Isolation Backends**: Docker containers, nsjail, and extensible architecture for additional backends
- **Runtime Support**: TypeScript, Bun, and Node.js with hot reload support
- **FaaS/Serverless API**: Deploy functions with automatic lifecycle management
- **Live File Updates**: Update code in running deployments with hot reload
- **RESTful API**: Clean HTTP API for sandbox and deployment management
- **Proxy Support**: Direct access to deployed web services
- **Security**: Network isolation, memory limits, CPU limits, and filesystem restrictions
- **Performance**: Fast startup times and efficient resource usage
- **Auto-scaling**: Automatic cleanup of idle deployments
- **Configurable**: Environment variables and config file support

## Quick Start

See `docs/HOW_TO_USE.md` for a full getting started guide.

### Prerequisites
- Docker (default backend)
- Rust toolchain

### Build and Run
```bash
cargo build --release
./target/release/sandbox-service
```

## API Usage

See `docs/API.md` for the full API reference and request/response examples.

### Health Check
```bash
curl http://localhost:8070/health
```

### Deploy a Bun FaaS Web Service (TypeScript or JavaScript)
```bash
curl -X POST http://localhost:8070/faas/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "runtime": "bun",
    "code": "import { serve } from \"bun\"; const server = serve({ port: 3000, fetch(req) { return new Response(\"Hello from Bun!\"); } });",
    "entry_point": "bun dev",
    "dev_server": false
  }'
```

### Advanced Bun FaaS Examples

See `tests/complex_bun_routing.json`, `tests/complex_bun_async.json`, `tests/complex_bun_env.json`, and `tests/complex_bun_fs.json` for:
- Multi-endpoint routing
- Async/await with external fetch
- Environment variable usage
- File system read/write

**All these advanced features are now fully supported!**

## Documentation

- All detailed guides, architecture docs, and workflow examples are in the `docs/` directory:
  - `docs/API.md` — Full API reference
  - `docs/HOW_TO_USE.md` — Getting started and usage guide
  - `docs/COMPLETE_TYPESCRIPT_WORKFLOW.md` — End-to-end TypeScript workflow
  - `docs/CONTAINER_LIFECYCLE_AND_FILE_TRANSFER.md` — Container/file management
  - `docs/PROXY_SOLUTION.md` — Proxy and networking details
  - `docs/TYPESCRIPT_PROJECT_GUIDE.md` — TypeScript project integration
  - `docs/ADMIN_UI_DEMO.md` — Admin UI usage
  - Example/test configs and scripts for FaaS and sandboxing

## Tests

- All test files are in `tests/` (see above for advanced Bun/Node/TypeScript FaaS tests)
- To run Rust tests:
```bash
cargo test
```

## License

MIT License - see LICENSE file for details.