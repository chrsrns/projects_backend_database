# Rust Profile Management Backend

Rust backend server I wrote for managing some of my online profile data. I intend to use this to centralize my profile information across my other projects. This uses a CLEAN architecture to make extensibility easier.

## Workspace layout

This repository is a Rust Cargo workspace with the following crates:

- **api**: Rocket HTTP layer
- **application**: Business logic
- **domain**: Core models + Diesel schema mapping
- **infrastructure**: Database connection + migrations
- **shared**: Shared response models and cross-cutting utilities
- **api_playground**: Leptos WASM interactive API explorer

## API server

The API is served by Rocket.

The primary binary is `src/main.rs` (CLI) which launches the Rocket server via `api::build_rocket()`.

All HTTP routes are mounted under the `/api` prefix.

### API Documentation

The API is documented using OpenAPI 3.0 and served via Swagger UI.

- Swagger UI: **GET** `/api/docs/`
- OpenAPI JSON: **GET** `/api/openapi.json`
- API Playground (interactive explorer): **GET** `/api_playground/`

### WebSocket (realtime resume change notifications)

- **GET** `/api/ws`

### Response format

Handlers wrap results using `shared::response_models::Response<T>` and serialize the wrapper to JSON.

### Markdown resume endpoints

Markdown bodies are accepted as `text/markdown` up to 1 MiB (larger payloads get a `413`).

- **GET** `/api/resume/{id}/export/markdown` — export a resume as Markdown (public resumes are anonymous, private need owner auth)
- **POST** `/api/resume/import/markdown` — import Markdown into a resume (Bearer auth)
- **GET** `/api/resume/markdown-format` — the Markdown format specification (anonymous)
- **POST** `/api/resume/validate/markdown` — validate Markdown against the parser (anonymous, no database access). Always `200` with `Response<MarkdownValidationReport>`: `{body: {valid, errors: [{section, message}]}}` (`section` may be `null`).
- **POST** `/api/resume/convert/markdown` — convert Markdown to a resume document (anonymous, no database access). `200` with `Response<ResumeDocumentEnvelope>`: `{body: {schema_version, generator, document}}`; `400` on invalid Markdown.
- **GET** `/api/resume/document-schema` — OpenAPI 3.0 components fragment for `ResumeDocument` (anonymous), used by external consumers such as renderers.

The resume document contract lives in `shared::resume_document`. `schema_version` is `1`; it is bumped only on breaking shape changes.

## Development

### Prerequisites

- Rust (edition 2024 compatible toolchain)
- PostgreSQL
- A `DATABASE_URL` environment variable (typically provided via a local `.env`)

### Database

- Migrations live under `infrastructure/migrations` (Diesel-style migration folders).

### Run the API

From the repository root:

```bash
cargo run -- serve
```

### Cross-Compilation

#### Cargo configuration

- `.cargo/config.toml` contains the Cargo settings that does not affect host compilation.
- `.cargo/config.aarch64.toml` contains additional environment configuration used only when cross-compiling for `aarch64-unknown-linux-gnu`.
    - Keeping the `aarch64` cross-compilation settings in a separate TOML file avoids interfering with native `cargo run` on x86-64 while still allowing cross-target builds when explicitly requested.

#### Host machine setup

On my Fedora machine, I needed to install the following packages to cross-compile for `aarch64-unknown-linux-gnu`:

```bash
sudo dnf install sysroot-aarch64-fc42-glibc.noarch

sudo dnf --installroot=/usr/aarch64-redhat-linux/sys-root/fc42 \
         --releasever=42 \
         --forcearch=aarch64 \
         --use-host-config \
         install sqlite-devel libgcc glibc-devel gcc libpq-devel

# BUG: Create development symlink for libgcc_s.so since it's missing in fedora:42
cd /usr/aarch64-redhat-linux/sys-root/fc42/usr/lib64
sudo ln -sf libgcc_s.so.1 libgcc_s.so
cd -
```

#### Using Docker for Cross-Compilation

A `Containerfile` is written to make it easier to cross-compile. To cross-compile a binary for aarch64 and on Domcloud's platform, run:

```bash
docker buildx build \
  --platform linux/arm64 \
  --target artifact \
  --output type=local,dest=./docker-build-out \
  -f Containerfile \
  .
```

## TODO

- [x] Setup basic project structure and architecture
- [x] Setup database schema and migrations
- [x] Setup database connection and Diesel integration
- [x] Setup API routes and basic handlers
- [x] Add tests to the `api` modules
- [x] Add a root-level binary purpose (CLI `serve` launches Rocket)
- [x] Add authentication handlers for `users` and `sessions` (schema/migrations already exist)
- [x] Add request/response examples for each endpoint (curl + JSON payloads)
- [ ] Add automated integration tests for full CRUD via HTTP
- [ ] Improve error handling: avoid `unwrap()` in handlers and standardize error payloads
- [ ] Add pagination/filtering/sorting for `GET /resumes`
