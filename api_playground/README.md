# API Playground

An interactive API explorer built with **Leptos** (Rust WebAssembly) for the Resume Profile Manager backend.

## Features

- **Auto-discovery**: Automatically fetches and parses your OpenAPI schema
- **Endpoint Browser**: Search and browse all API endpoints grouped by tags
- **Interactive Request Builder**: Dynamic forms for path params, query params, and request bodies
- **Live Response Viewer**: See formatted JSON responses with syntax highlighting
- **Dark Mode UI**: Modern, developer-friendly dark theme

## Architecture

```
api_playground/
├── Cargo.toml          # Leptos CSR dependencies
├── index.html          # App shell with CSS
├── build.sh            # Build script
├── src/
│   ├── main.rs         # Entry point
│   ├── app.rs          # Main App component
│   ├── components/
│   │   ├── endpoint_browser.rs    # Sidebar endpoint list
│   │   ├── request_builder.rs     # Request form
│   │   └── response_viewer.rs     # Response display
│   └── services/
│       └── api_client.rs          # HTTP client + OpenAPI parser
└── dist/               # Build output (generated)
```

## Build Instructions

### Prerequisites

- Rust with `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` matching the `wasm-bindgen` crate version

```bash
# Install wasm-bindgen-cli (match version in Cargo.toml)
cargo install -f wasm-bindgen-cli --version 0.2.104
```

### Build

```bash
# Using the build script
./build.sh

# Or manually:
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --no-typescript \
    --out-dir dist \
    ../target/wasm32-unknown-unknown/release/api_playground.wasm
cp index.html dist/
```

### Serve

```bash
# Serve the dist folder locally
python3 -m http.server 9000 --directory dist

# Or use any static file server
```

Then open `http://localhost:9000` in your browser.

## Integration with Backend

The playground expects your Rocket backend to be running (default: `http://localhost:8000`) and serving the OpenAPI schema at `/api/openapi.json`.

To serve this from your Rocket backend, copy the `dist/` contents to your static file serving directory, or use Rocket's `FileServer` to serve the files.

## Usage

1. Enter your backend base URL (default: `http://localhost:8000`)
2. Click "Refresh Endpoints" to load the API schema
3. Browse endpoints by tag in the sidebar
4. Click an endpoint to see its details
5. Fill in parameters and request body
6. Click "Send Request" to execute
7. View the formatted response

## Technology Stack

- **Leptos 0.7** - Rust frontend framework
- **gloo-net** - HTTP client for WASM
- **wasm-bindgen** - Rust/JavaScript bindings
- **web-sys** - Web API bindings
- **serde_json** - JSON parsing
