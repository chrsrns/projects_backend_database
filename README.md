# projects_backend_database

Rust backend service for managing some profile data. This uses a CLEAN architecture to make extensibility easier.

## Workspace layout

This repository is a Rust Cargo workspace with the following crates:

- **api**: Rocket HTTP layer
- **application**: Business logic
- **domain**: Core models + Diesel schema mapping
- **infrastructure**: Database connection + migrations
- **shared**: Shared response models and cross-cutting utilities

## API server

The API is served by Rocket.

The primary binary is `src/main.rs` (CLI) which launches the Rocket server via `api::build_rocket()`.

All HTTP routes are mounted under the `/api` prefix.

### API Documentation

The API is documented using OpenAPI 3.0 and served via Swagger UI.

- Swagger UI: **GET** `/api/docs/`
- OpenAPI JSON: **GET** `/api/openapi.json`

### WebSocket (realtime resume change notifications)

- **GET** `/api/ws`

### Response format

Handlers wrap results using `shared::response_models::Response<T>` and serialize the wrapper to JSON.

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
