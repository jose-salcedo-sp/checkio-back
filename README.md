# checkio-back

REST API for a restaurant-style operations backend: staff accounts, RBAC metadata, dining tables, menu categories/items, and orders. Built with **Axum**, **SQLx** (SQLite), and **JWT** bearer authentication.

## Tech stack

| Layer | Choice |
|--------|--------|
| Runtime | Tokio |
| HTTP | Axum 0.8 |
| Database | SQLite via SQLx (macros + migrations) |
| Auth | Argon2 password hashes, JWT (`jsonwebtoken`) |
| Config | `dotenvy` + environment variables |
| Observability | `tracing` + `tower-http` request traces |

Development builds use **offline SQLx** metadata from the checked-in `.sqlx/` directory (see [`.cargo/config.toml`](.cargo/config.toml)), so `cargo check` does not need a live database.

## Repository layout

```text
back/
├── Cargo.toml              # Crate manifest & dependencies
├── .cargo/config.toml      # SQLX_OFFLINE for compile-time query validation
├── .sqlx/                  # Cached query fingerprints (offline mode)
├── migrations/             # Ordered SQL migrations (sqlx::migrate)
├── dev.db                  # Local SQLite file (if used; not required in CI)
└── src/
    ├── main.rs             # Entry: tracing, config, CORS, TCP listen, serve router
    ├── config.rs           # HOST, PORT, DATABASE_URL, JWT_SECRET, CLIENT_ORIGIN
    ├── db.rs               # SQLite pool (foreign keys on) + migrate helper
    ├── auth.rs             # JWT claims encode/decode
    ├── error.rs            # AppError + JSON error responses
    ├── migrations.rs       # Alternate pool/migrate (duplicate of db helpers)
    ├── routes/
    │   ├── mod.rs          # App router: public auth + JWT-protected API nests
    │   ├── auth.rs         # POST /api/auth/login
    │   ├── users.rs
    │   ├── tables.rs
    │   ├── orders.rs
    │   ├── menu_items.rs
    │   └── menu_item_categories.rs
    └── utils/
        ├── auth.rs         # Axum extractor: Bearer JWT → AuthUser { user_id, role }
        └── passwords.rs    # Argon2 verify helper
```

## Architecture (high level)

```mermaid
flowchart LR
    subgraph client [Client]
        WebOrMobile[Web / mobile app]
    end

    subgraph server [checkio-back]
        Axum[Axum router]
        Cors[CORS layer]
        Trace[TraceLayer]
        AuthN[JWT middleware / extractor]
        Handlers[Route handlers]
    end

    subgraph data [Data]
        SQLite[(SQLite)]
    end

    WebOrMobile --> Cors --> Trace --> Axum
    Axum --> AuthN
    Axum --> Handlers
    Handlers --> SQLite
```

## Request flow: public vs protected

```mermaid
flowchart TB
    subgraph public [Public]
        Login[POST /api/auth/login]
        Login --> Pool[(SqlitePool)]
        Login --> Token[Plain-text JWT string in body]
    end

    subgraph protected [Protected under /api/*]
        MW[route_layer AuthUser extractor]
        MW --> H[Handlers]
        H --> Pool2[(SqlitePool)]
    end

    Client[Client] --> Login
    Client -->|"Authorization: Bearer ..."| MW
```

Protected routes apply `middleware::from_extractor::<AuthUser>()`, which reads `JWT_SECRET` from the environment, parses the `Authorization: Bearer` header, and injects `AuthUser { user_id, role }` for handlers that need it.

> **RBAC in the database:** Migrations seed `roles`, `permission_resources`, `permissions`, and `role_permissions`. The JWT includes a **role slug**, but application handlers do not yet enforce fine-grained permission checks against those tables—that is natural follow-up work if you need server-side authorization beyond “valid logged-in user.”

## HTTP API surface

| Method | Path | Auth |
|--------|------|------|
| `POST` | `/api/auth/login` | No |
| `GET`, `POST` | `/api/users`, `/api/users/{id}` | JWT |
| `GET`, `POST` | `/api/menu_items`, `/api/menu_items/{id}` | JWT |
| `GET`, `POST` | `/api/menu_item_categories`, `/api/menu_item_categories/{id}` | JWT |
| `GET`, `POST` | `/api/orders`, `/api/orders/{id}` | JWT |
| `GET`, `POST` | `/api/tables`, `/api/tables/{id}` | JWT |
| `PATCH` | `/api/tables/{id}` | JWT |

CORS allows `GET`, `POST`, `PUT`, `PATCH` from the configured `CLIENT_ORIGIN`.

## Data model (conceptual)

```mermaid
erDiagram
    roles ||--o{ users : assigns
    users ||--o{ orders : creates
    tables ||--o{ orders : hosts
    orders ||--o{ order_items : contains
    menu_categories ||--o{ menu_items : groups
    users ||--o{ menu_categories : creates
    users ||--o{ menu_items : creates

    roles {
        text id PK
        text slug UK
        text name
    }

    users {
        text id PK
        text email UK
        text role_id FK
        text password_hash
    }

    tables {
        text id PK
        text label UK
    }

    orders {
        text id PK
        text table_id FK
        text status
        text created_by_id FK
    }

    order_items {
        text id PK
        text order_id FK
        text menu_item_id FK
        int quantity
        real unit_price
    }

    menu_categories {
        text id PK
        text name UK
    }

    menu_items {
        text id PK
        text category_id FK
        real price
    }

    permission_resources ||--o{ permissions : scopes
    roles ||--o{ role_permissions : grants
    permissions ||--o{ role_permissions : granted
```

RBAC tables (`permission_resources`, `permissions`, `role_permissions`) complement `roles` and are populated in [`migrations/20260506203520_create_rbac.sql`](migrations/20260506203520_create_rbac.sql).

## Configuration

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | SQLite URL (e.g. `sqlite:dev.db` or `sqlite::memory:`) |
| `JWT_SECRET` | Symmetric key for signing and verifying tokens |
| `CLIENT_ORIGIN` | Allowed browser origin for CORS (parsed as `HeaderValue`) |
| `HOST` | Bind address (default `0.0.0.0`) |
| `PORT` | TCP port (default `3000`) |

Logging uses `RUST_LOG` when set; otherwise defaults include `checkio_back=debug` and related crates.

## Running locally

1. Copy or create a `.env` with the variables above (a `DATABASE_URL` pointing at a file or in-memory DB).
2. Apply migrations to that database (for example with [`sqlx-cli`](https://github.com/launchbadge/sqlx): `sqlx database create` / `sqlx migrate run` from this directory, or by wiring `db::run_migrations` into startup if you prefer).
3. `cargo run`

The server prints the listening address when it starts.

**Migration order:** SQLx runs files in lexical version order. `menu_items` references `menu_categories`, so the categories migration filename must sort **before** the menu-items migration on a fresh database. If `sqlx migrate run` fails on `menu_items`, adjust the migration timestamps (or merge migrations) so categories are created first.

---

*Diagrams use [Mermaid](https://mermaid.js.org/); they render in GitHub, GitLab, and many Markdown viewers.*
