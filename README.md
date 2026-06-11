# base_app

**Overview**
- **Project**: base_app — simple Axum-based backend for campaign management and auth.
- **Language**: Rust (Tokio + Axum).

**Quick Start**
- Using Docker Compose (recommended):

```bash
docker compose up -d --build
```

- Local development (requires Rust toolchain):

```bash
cargo build
cargo run --bin base_app
```

**Environment variables**
- **PORT**: HTTP port the server listens on (default `8000`).
- **FRONTEND_URL**: URL allowed by CORS (default `http://localhost:5173`).
- **DATABASE_URL**: Postgres connection string used by the app.
- **POSTGRES_USER / POSTGRES_PASSWORD / POSTGRES_DB**: used by the `db` service when running locally via docker-compose.

Place variables in a `.env` file or supply via your environment/docker-compose.

**Docker**
- The repo contains a `Dockerfile` and `docker-compose.yml` for local development.
- Build & run:

```bash
docker compose up --build
```

Notes: The repository Dockerfile builds a release binary and packages it into a minimal Debian image.

**API**
- See full endpoint list in [docs/API.md](docs/API.md).

**Project layout**
- `src/main.rs` — HTTP server, routing and handlers.
- `src/` — other modules (db.rs, engine.rs, routes, api, security).
- `Dockerfile` — multi-stage build for release binary.
- `docker-compose.yml` — db + api services for local development.

**Development notes**
- Run `cargo build` to verify compilation; CI should run `cargo test` where applicable.
- The app uses `dotenvy` — create a `.env` file for local env vars.

**Next steps**
- Add database migrations (e.g., with `sqlx migrate`) and tests for API handlers.

---

Created files:
- [README.md](README.md)
- [docs/API.md](docs/API.md)

If you want, I can add a quick `Makefile` or GitHub Actions workflow next.