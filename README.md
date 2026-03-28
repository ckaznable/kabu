# Kabu

Personal stock portfolio tracker. Tracks holdings, fetches live prices from Finnhub, and extracts transactions from PDF financial documents using Gemini LLM.

## Architecture

```
kabu/
├── server/      — Axum HTTP server (API + static frontend)
├── updater/     — One-shot price updater binary (for K8s CronJob)
├── shared/      — Shared library (DB, models, config)
├── typegen/     — TypeScript type generator (Rust → TS via specta)
├── frontend/    — Vue 3 + Vite SPA
└── migrations/  — SQL schema
```

## Configuration

Copy `config.example.toml` to `config.toml` and fill in your API keys:

```bash
cp config.example.toml config.toml
```

### config.toml

| Section    | Field           | Description                                         |
|------------|-----------------|-----------------------------------------------------|
| `server`   | `port`          | HTTP server port (default: `3000`)                  |
| `server`   | `database_url`  | SQLite connection string (default: `sqlite:kabu.db`)|
| `finnhub`  | `api_key`       | Finnhub API key (direct value)                      |
| `finnhub`  | `api_key_env`   | Or: env var name to read Finnhub key from           |
| `gemini`   | `api_key`       | Gemini API key (direct value)                       |
| `gemini`   | `api_key_env`   | Or: env var name to read Gemini key from            |
| `gemini`   | `model`         | Gemini model name (default: `gemini-2.5-flash-lite`)|
| `exchange_rate` | `api_key`   | ExchangeRate-API key (direct value)                 |
| `exchange_rate` | `api_key_env` | Or: env var name for ExchangeRate-API key         |
| `exchange_rate` | `base`      | Base currency code (default: `USD`)                 |
| `exchange_rate` | `currencies`| Target currency codes to store                       |
| `pdf`      | `password`      | PDF decryption password (direct value)              |
| `pdf`      | `password_env`  | Or: env var name to read PDF password from          |

For each secret, you can either:
- Set the value directly in the config file, or
- Set the `_env` variant to the name of an environment variable (e.g. `FINNHUB_API_KEY`), and export that variable in your shell

Config file path can be overridden with the `KABU_CONFIG` env var.

### Required API Keys

- **Finnhub** — Get a free key at https://finnhub.io/
- **Gemini** — Get a key at https://aistudio.google.com/apikey
- **ExchangeRate-API** — Get a key at https://www.exchangerate-api.com/

## Development

### Prerequisites

- Rust (stable, edition 2024)
- Node.js 18+
- Make

### Using Make

```bash
make              # Build everything (frontend + server + updater)
make dev          # Build frontend, then run server
make dev-frontend # Vite dev server on http://localhost:5173 (proxies /api to :3000)
make dev-server   # Run Rust server directly
make typegen      # Regenerate TypeScript types from Rust models
make lint         # Run oxlint on frontend
make clean        # Remove all build artifacts
```

### Manual

```bash
# Backend
cargo build
cargo run -p kabu-server
cargo run -p kabu-updater   # One-shot: fetches latest prices then exits

# Frontend
cd frontend
npm install
npm run dev
npm run build    # Production build → frontend/dist/
```

### Type Generation

Shared types between Rust and TypeScript are generated with specta:

```bash
make typegen   # Outputs frontend/src/api/types.ts
```

## Production / K8s

Build two separate binaries for deployment:

```bash
make build
# or
cargo build --release -p kabu-server
cargo build --release -p kabu-updater
```

- `kabu-server` — serves the API and frontend static files from `frontend/dist/`
- `kabu-updater` — one-shot binary, fetches prices for all tracked stocks then exits. Schedule with K8s CronJob for post-market updates.

Both binaries read from `config.toml` (or the path in `KABU_CONFIG`).

## API Endpoints

| Method   | Path                     | Description                                 |
|----------|--------------------------|---------------------------------------------|
| `GET`    | `/api/stocks`            | List all tracked stocks                     |
| `POST`   | `/api/stocks`            | Add a stock to track                        |
| `GET`    | `/api/stocks/{id}`       | Get stock details                           |
| `PUT`    | `/api/stocks/{id}`       | Update stock quantity / cost                |
| `DELETE` | `/api/stocks/{id}`       | Remove a stock                              |
| `GET`    | `/api/portfolio/summary` | Portfolio overview with prices              |
| `POST`   | `/api/pdf/upload`        | Upload PDF, extract transactions via Gemini |
| `GET`    | `/api/transactions`      | List extracted transactions                 |
