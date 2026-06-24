# API Playground

An interactive API explorer built with **Leptos** (Rust WebAssembly) for the Resume Profile Manager backend.

## Features

- **Auto-discovery**: Automatically fetches and parses your OpenAPI schema
- **Endpoint Browser**: Search and browse all API endpoints grouped by tags, with selected endpoint highlighting
- **Interactive Request Builder**: Dynamic forms for path params, query params, and request bodies, with a live JSON payload preview
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
│   │   ├── response_viewer.rs     # Response display
│   │   ├── schema_form.rs         # Dynamic form generator from JSON Schema
│   │   └── mod.rs                 # Component module exports
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

The Rocket backend automatically mounts a `FileServer` at `/api_playground` that serves files directly from `api_playground/dist/`. Just run `./build.sh` and start the backend — no manual copying or extra configuration is needed.

## Usage

*The playground loads the API schema automatically on startup, but if it fails to load, you can manually specify the backend URL.*

1. Browse endpoints by tag in the sidebar
2. Click an endpoint to see its details
3. Fill in parameters and request body
4. Click "Send Request" to execute
5. View the formatted response

## Technology Stack

- **Leptos 0.7** - Rust frontend framework
- **gloo-net** - HTTP client for WASM
- **wasm-bindgen** - Rust/JavaScript bindings
- **web-sys** - Web API bindings
- **serde_json** - JSON parsing
