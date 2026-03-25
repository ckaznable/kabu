# Kabu

Personal stock portfolio tracker. Tracks holdings, fetches live prices from Finnhub, and extracts transactions from PDF financial documents using Gemini LLM.

## Architecture

```
kabu/
├── server/      — Axum HTTP server (API + static frontend)
├── updater/     — Standalone price updater binary (for K8s)
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
| `gemini`   | `model`         | Gemini model name (default: `gemini-2.0-flash`)     |
| `updater`  | `interval_secs` | Price update interval in seconds (default: `300`)   |

For each API key, you can either:
- Set `api_key` directly in the config file, or
- Set `api_key_env` to the name of an environment variable (e.g. `FINNHUB_API_KEY`), and export that variable in your shell

Config file path can be overridden with the `KABU_CONFIG` env var.

### Required API Keys

- **Finnhub** — Get a free key at https://finnhub.io/
- **Gemini** — Get a key at https://aistudio.google.com/apikey

## Development

### Prerequisites

- Rust (stable)
- Node.js 18+

### Backend

```bash
# Build all Rust binaries
cargo build

# Run the server
cargo run -p kabu-server

# Run the updater (separate process)
cargo run -p kabu-updater
```

### Frontend

```bash
cd frontend
npm install
npm run dev      # Dev server on http://localhost:5173 (proxies /api to :3000)
npm run lint     # Lint with oxlint
npm run build    # Production build → frontend/dist/
```

### Type Generation

Shared types between Rust and TypeScript are generated with specta:

```bash
cargo run -p kabu-typegen   # Outputs frontend/src/api/types.ts
```

## Production / K8s

Build two separate binaries for deployment:

```bash
cargo build --release -p kabu-server
cargo build --release -p kabu-updater
```

- `kabu-server` — serves the API and frontend static files from `frontend/dist/`
- `kabu-updater` — runs as a separate pod, periodically fetches prices from Finnhub

Both binaries read from `config.toml` (or the path in `KABU_CONFIG`).

## API Endpoints

| Method   | Path                  | Description                        |
|----------|-----------------------|------------------------------------|
| `GET`    | `/api/stocks`         | List all tracked stocks            |
| `POST`   | `/api/stocks`         | Add a stock to track               |
| `GET`    | `/api/stocks/:id`     | Get stock details                  |
| `PUT`    | `/api/stocks/:id`     | Update stock quantity / cost       |
| `DELETE` | `/api/stocks/:id`     | Remove a stock                     |
| `GET`    | `/api/portfolio/summary` | Portfolio overview with prices  |
| `POST`   | `/api/pdf/upload`     | Upload PDF, extract transactions via Gemini |
| `GET`    | `/api/transactions`   | List extracted transactions        |
