# projects_backend_database

Rust backend service for managing resume/CV data using a Clean Architecture / layered approach.

## Workspace layout

This repository is a Cargo workspace with the following crates:

- **api**: Rocket HTTP layer (request routing / handlers)
- **application**: Use-cases / business logic (CRUD orchestration)
- **domain**: Core models + Diesel schema mapping
- **infrastructure**: Database connection + migrations
- **shared**: Shared response models and cross-cutting utilities

## API server

The Rocket server is defined in `api/src/bin/main.rs` and mounts routes under the `/api` prefix.

### Endpoints (Resume CRUD)

All routes are prefixed with `/api`.

- **GET** `/resumes`
  - Returns all resumes
- **GET** `/resume/<resume_id>`
  - Returns a single resume by id
- **POST** `/new_resume`
  - Creates a resume
  - `Content-Type: application/json`
- **PUT** `/resume/<resume_id>`
  - Updates a resume
  - `Content-Type: application/json`
- **DELETE** `/resume/<resume_id>`
  - Deletes a resume

### Response format

Handlers wrap results using `shared::response_models::{Response, ResponseBody}` and serialize the wrapper to JSON.

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
- [x] Add a root-level binary purpose (current `src/main.rs` is a placeholder)
- [ ] Add authentication handlers for `users` and `sessions` (schema/migrations already exist)
- [ ] Add request/response examples for each endpoint (curl + JSON payloads)
- [ ] Add automated integration tests for full CRUD via HTTP
- [ ] Improve error handling: avoid `unwrap()` in handlers and standardize error payloads
- [ ] Add pagination/filtering/sorting for `GET /resumes`
