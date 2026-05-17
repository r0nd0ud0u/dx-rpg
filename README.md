# Dx-rpg

A browser-based multiplayer RPG built with [Dioxus](https://dioxuslabs.com/) (Rust fullstack framework) using the [lib-rpg](https://github.com/r0nd0ud0u/lib-rpg) engine for game logic.

- **Frontend**: Dioxus (WebAssembly)
- **Backend**: Axum (HTTP + WebSocket server)
- **Database**: SQLite via SQLx
- **Auth**: Session-based with axum-session

---

## Table of Contents

- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Data Flow](#data-flow)
- [Development Setup](#development-setup)
- [Deployment](#deployment)
- [Screenshots](#screenshots)

---

## Features

### 10-Stage Scenario Progression

The game ships with 10 progressive stages designed to level up players gradually:

| Stage | Name | Enemy | Recommended Level |
|-------|------|-------|-------------------|
| 1 | Patrouille Gobeline | Gobelin Eclaireur | 1 |
| 2 | Embuscade Gobeline | Gobelin Eclaireur + Angmar10PV | 2 |
| 3 | Le Pillard Orc | Orc Pillard | 3 |
| 4 | Patrouille Mixte | Orc Pillard + Gobelin Eclaireur | 4 |
| 5 | Le Champion des Orcs | Champion Orc | 5 |
| 6 | Colère du Champion | Champion Orc + Gobelin Eclaireur | 6 |
| 7 | L'Ombre du Mordor | Nécromancien du Mordor | 7 |
| 8 | Rituel des Ténèbres | Nécromancien + Gobelin Eclaireur | 8 |
| 9 | Le Cavalier Sans Visage | Nazgul | 9 |
| 10 | L'Oeil de Sauron | Sauron l'Oeil Flamboyant (Boss) | 10 |

Each scenario is defined as a JSON file under `offlines/scenarios/`. Enemies are defined in `offlines/characters/` and their attacks in `offlines/attack/<enemy-name>/`.

### Save Slot System

Players are limited to a configurable number of save slots (default: 3). Configure via `.env`:

```env
MAX_SAVES=3
```

The "Create Server" page shows all save slots with:
- Empty slots: click to start a new game immediately
- Occupied slots: shows game name, current scenario, and last save date; click to select then "Overwrite & Play"

### Attack Tooltips with Description

Attacks can now include an optional `Description` field in their JSON:

```json
{
  "Name": "Griffure",
  "Description": "A quick scratch dealing light physical damage.",
  ...
}
```

When present, hovering over the attack button in the character sheet shows the description as a tooltip.

This requires `lib-rpg` branch `feat/attack-description` (commit `08af5ad82d406aee8f452f05e7b763035fa2fd44`).

### Scenarios Progress Sheet

During a game, click **📜 Scenarios** in the game toolbar to open a side sheet showing all scenarios and their progress state (Not Started / In Progress / ✅ Completed).

### Admin Panel

Enable the admin panel via `.env`:

```env
ADMIN_ENABLED=true
```

Navigate to `/admin` to access:
- **Users tab**: list all users with connection status and save count; delete users
- **Scenarios tab**: list all loaded scenarios with level, boss count, and description

---

## Configuration (`.env`)

| Variable | Default | Description |
|----------|---------|-------------|
| `IP` | `0.0.0.0` | Bind address |
| `PORT` | `8080` | HTTP port |
| `DATABASE_URL` | `sqlite:db.sqlite` | SQLite connection string |
| `USE_PASSWORD` | `false` | Require password on login |
| `MAX_SAVES` | `3` | Max save slots per user |
| `ADMIN_ENABLED` | `false` | Enable `/admin` panel |

---

## Architecture

### High-level overview

```mermaid
graph TD
    Browser["Browser (WASM client)"]
    Nginx["Nginx (reverse proxy + SSL)"]
    App["dx-rpg server (Axum + Dioxus)"]
    DB[("SQLite\n/data/db.sqlite")]
    FS[("File system\nsaved_data/")]
    LibRpg["lib-rpg\n(game engine)"]

    Browser -- "HTTPS + WSS" --> Nginx
    Nginx -- "HTTP proxy :8080" --> App
    App -- "SQLx" --> DB
    App -- "fs::read/write" --> FS
    App -- "Rust crate" --> LibRpg
```

### Module map

```mermaid
graph LR
    subgraph Frontend["Frontend (WASM)"]
        Pages["board_game_components/\nhome · lobby · gameboard\ncharacter · admin · login"]
        Components["components/\nbutton · input · select\ndialog · popover · tabs"]
        Widgets["widgets/\ncharts · tab_equipment\nalert_dialog"]
    end

    subgraph Backend["Backend (server feature)"]
        Auth["auth_manager/\nauth · db · server_fn\nmodel · session"]
        WS["websocket_handler/\nevent · msg_from_client\nevent_inventory"]
        Utils["utils/\nserver_file_utils"]
    end

    subgraph Shared["Shared (both targets)"]
        Common["common.rs\nglobal signals · routes\nconstants"]
    end

    Pages --> Common
    Pages --> WS
    Auth --> Common
    WS --> Common
    WS --> Utils
```

---

## Project Structure

```
dx-rpg/
├── src/
│   ├── main.rs                   # Entry point — client launch + server setup (Axum)
│   ├── lib.rs                    # Module declarations
│   ├── common.rs                 # Shared globals (signals, routes, constants)
│   ├── auth_manager/             # User authentication & session management (server only)
│   │   ├── db.rs                 # SQLite pool init, table creation, seed data
│   │   ├── auth.rs               # Axum auth layer
│   │   ├── server_fn.rs          # Dioxus server functions for login/logout
│   │   └── model.rs              # User model
│   ├── board_game_components/    # Page-level Dioxus components
│   │   ├── home_page.rs          # Landing / dashboard
│   │   ├── login_page.rs         # Login form
│   │   ├── lobby_page.rs         # Pre-game lobby
│   │   ├── gameboard.rs          # Main game UI
│   │   ├── character_page.rs     # Character sheet
│   │   ├── admin_page.rs         # Admin panel
│   │   └── ...
│   ├── components/               # Reusable UI primitives (button, input, select…)
│   ├── websocket_handler/        # Real-time game event bus (client ↔ server)
│   │   ├── event.rs              # ClientEvent / ServerEvent enums + handlers
│   │   ├── msg_from_client.rs    # Incoming message parsing
│   │   └── event_inventory.rs    # Per-session event queue
│   ├── utils/
│   │   └── server_file_utils.rs  # Save / load game state to/from disk
│   └── widgets/                  # Composite UI widgets (charts, equipment tab…)
├── offlines/                     # Static game data (characters, scenarios, attacks)
│   ├── characters/               # JSON character definitions
│   ├── scenarios/                # JSON scenario definitions
│   └── attack/                   # JSON attack/skill definitions
├── assets/                       # CSS and static assets
├── docs/                         # Deployment documentation
├── scripts/                      # Build & Docker helper scripts
├── Dockerfile                    # Multi-stage Docker build
├── docker-compose.yml            # Production stack (app + SQLite web UI)
└── Cargo.toml
```

---

## Data Flow

### Authentication flow

```mermaid
sequenceDiagram
    actor User
    participant Browser
    participant Server
    participant SQLite

    User->>Browser: Enter credentials
    Browser->>Server: POST /api/login (server function)
    Server->>SQLite: SELECT user WHERE username=?
    SQLite-->>Server: User row + permissions
    Server-->>Browser: Set session cookie
    Browser-->>User: Redirect to home
```

### Game session flow (WebSocket)

```mermaid
sequenceDiagram
    actor GM as Game Master
    actor Player
    participant Server
    participant LibRpg as lib-rpg engine
    participant FS as saved_data/

    GM->>Server: WS connect + CreateServer event
    Server->>LibRpg: init_server()
    Player->>Server: WS connect + JoinGame event
    Server-->>Player: ServerEvent::GameState

    loop Game turn
        GM->>Server: ClientEvent::Attack(...)
        Server->>LibRpg: resolve_attack()
        LibRpg-->>Server: updated GameState
        Server->>FS: save_game_state()
        Server-->>GM: ServerEvent::Update
        Server-->>Player: ServerEvent::Update
    end
```

### Docker / production deployment flow

```mermaid
graph LR
    Internet -->|HTTPS :443| Nginx
    Nginx -->|HTTP :8080| AppContainer["dx-rpg container"]
    AppContainer -->|sqlite:///data/db.sqlite| DBVol[("Docker volume\ndb_data")]
    AppContainer -->|fs write| SaveVol[("Docker volume\nsaved_data")]
    Admin -->|SSH tunnel :8082| SqliteWeb["sqlite-web container\n(read-only UI)"]
    SqliteWeb -->|read-only| DBVol
```

---

## Development Setup

### Prerequisites

- Rust stable toolchain
- Dioxus CLI (`cargo binstall dioxus-cli@0.7.9 --force`)
- SSH access to the lib-rpg private dependency (add to `~/.cargo/config.toml`):

```toml
[net]
git-fetch-with-cli = true

[target.wasm32-unknown-unknown]
rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
```

### Run locally

```bash
dx serve --platform web
# Open http://localhost:8080
```

---

## Deployment

### With Docker Compose (recommended for production)

```bash
# Pull and start (app + SQLite web UI)
./scripts/docker_compose_up.sh

# Stop (data volumes are preserved)
./scripts/docker_compose_down.sh

# Full reset including data
docker compose down -v
```

**Persistent data** is stored in two named Docker volumes:

| Volume | Path in container | Content |
|--------|-------------------|---------|
| `dx-rpg_db_data` | `/data/db.sqlite` | User accounts, sessions |
| `dx-rpg_saved_data` | `/usr/local/app/saved_data/` | Per-user game saves |

Both volumes survive `docker compose stop`, `docker compose down`, and image updates. Only `docker compose down -v` removes them.

### SQLite web UI (admin)

The `sqlite-web` service runs on port **8082**, bound to loopback only.  
Access it remotely via SSH tunnel:

```bash
ssh -L 8082:localhost:8082 user@your-server
# Then open http://localhost:8082 in your browser
```

### Build locally and push

```bash
# Build image locally
./scripts/docker_build.sh

# Or trigger the GitHub Action by pushing a tag
git tag v1.2.3 && git push origin v1.2.3
```

---

## Screenshots

### Home page — before login
<img width="1909" height="431" alt="image" src="https://github.com/user-attachments/assets/09e2e271-29fa-4d1a-a13c-4fb26255f2b4" />

### Home page — after login
<img width="1904" height="346" alt="image" src="https://github.com/user-attachments/assets/bc050e92-1b10-4293-ac90-4dc8e706e622" />

### Create server page
<img width="1899" height="253" alt="image" src="https://github.com/user-attachments/assets/073c230f-7344-48d0-b75f-75121b36f2d2" />

### New game page
<img width="1911" height="268" alt="image" src="https://github.com/user-attachments/assets/4b7284e2-ee13-4ff8-9ac1-31742ed2cf1a" />

### Load game page
<img width="1903" height="416" alt="image" src="https://github.com/user-attachments/assets/86e63864-2df6-4862-a95d-fe3eb41422e9" />

### Join game page
<img width="1916" height="189" alt="image" src="https://github.com/user-attachments/assets/541fd928-f228-4c35-a7d4-a4ca86b3b5a2" />

