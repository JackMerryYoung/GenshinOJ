# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RsOJ is an Online Judge platform that uses WebSocket for real-time communication between client and server. The project has migrated from a Python-based backend to a Rust-based modular architecture while maintaining a React/TypeScript frontend.

## Architecture

### Modular Plugin System

The backend uses a **working load** architecture where modules are compiled as dynamic libraries (.so on Linux, .dll on Windows) and loaded at runtime by `main_backend`. Each module:

- Implements a specific protocol (e.g., `std_ws_server@0.1.0`, `std_judge@0.1.0`)
- Exports `on_init()` and `on_unload()` FFI functions
- Runs in its own Tokio runtime
- Can depend on other modules via protocol versioning

Module loading follows topological sort based on dependencies defined in `module_config_rs.json`. The system supports:
- **Restricted mode**: Any module panic shuts down the entire server
- **Dependency resolution**: Modules load only after their dependencies initialize
- **Protocol versioning**: Dependencies specify required protocol versions

### Core Modules

Located in `rust_backend/`:

- **main_backend**: Entry point that loads and orchestrates all modules
- **ws_server**: WebSocket server handling client connections on port 9983
- **simple_authenticator**: Manages user authentication and session tokens
- **judge**: Code submission judging, problem management, solutions, discussions
- **chat_server**: Real-time chat functionality
- **userish**: User profiles, follow/unfollow, friends list
- **control_panel**: Administrative interface
- **db_connector**: MySQL database connection pooling

### WebSocket Message Protocol

Client-server communication follows a convention where incoming messages trigger handlers named `on_<command>`. For example:

- Client sends `{"command": "login", "content": {...}}`
- Server dispatches to `on_login()` in each module that implements it
- Modules register external listeners in `ws_server_config_rs.json`

This allows modules to extend functionality without modifying the core WebSocket server.

### Database

MySQL database named `RsOJ` with tables for:
- Users (authentication managed by `simple_authenticator`)
- Submissions, problems, solutions
- Discussions, replies, votes
- Notifications
- User relationships (follows, friends)

Tables are auto-created on module initialization via `CREATE TABLE IF NOT EXISTS`.

## Build and Run

### Backend (Rust)

**Build all modules:**
```bash
cd rust_backend
./build&run.sh
```

This script:
1. Builds each module package (`-p <module_name>`) in release mode
2. Moves compiled `.so` files to `rust_backend/modules/<module_name>/`
3. Runs `main_backend` which loads modules from `module_config_rs.json`

**Debug mode:**
```bash
cd rust_backend
./debug.sh
```

**Module configuration:** Edit `module_config_rs.json` to enable/disable modules or change dependencies.

**Database setup:** MySQL must be running. Default credentials: `root:123456@localhost:3306`. Override with environment variables: `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`.

**Clear database:**
```bash
python clear_database.py       # prompts for confirmation
python clear_database.py -y    # auto-confirm
```

### Frontend (React + Vite)

**Development server:**
```bash
cd client_web
npm install
npm run dev
```

Runs on `http://0.0.0.0:5173` with HMR enabled and proxies:
- `/wsapi` → `ws://localhost:9983/ws` (WebSocket)
- `/avatar` → `http://localhost:9983` (static files)

**Production build:**
```bash
cd client_web
npm run build
npm run preview
```

**Lint:**
```bash
cd client_web
npm run lint
```

## Development Workflow

### Adding a New WebSocket Command

1. **Register command** in `rust_backend/ws_server/ws_server_config_rs.json`:
   ```json
   {"command": "on_your_command", "required_protocol_version": "*"}
   ```

2. **Implement handler** in the relevant module (e.g., `judge/src/socket_actions/on_your_command.rs`):
   ```rust
   pub async fn on_your_command(
       ws_id: String,
       content: serde_json::Value
   ) -> Result<(), Box<dyn std::error::Error>> {
       // Implementation
   }
   ```

3. **Export from module** in `socket_actions/mod.rs`:
   ```rust
   pub mod on_your_command;
   ```

4. **Frontend integration** - send message via WebSocket:
   ```typescript
   sendJsonMessage({
       command: "your_command",
       content: { /* data */ }
   });
   ```

### Adding a New Module

1. Create module in `rust_backend/<module_name>/`
2. Implement `on_init()` and `on_unload()` with `#[unsafe(no_mangle)]`
3. Add to workspace in `rust_backend/Cargo.toml`
4. Configure in `module_config_rs.json` with appropriate dependencies
5. Update `build&run.sh` to build and move the module

### Rust Compiler Configuration

The workspace uses nightly features:
- `thread_id_value` for logging
- Cranelift codegen backend in dev mode (`codegen-backend = "cranelift"`)
- Parallel compilation (`-Zthreads=8`)
- LTO in release mode

## Frontend Stack

- **React 19** with React Router for routing
- **Fluent UI** components (`@fluentui/react-components`)
- **Monaco Editor** for code editing
- **React Markdown** with KaTeX for math rendering
- **Redux Toolkit** for state management
- **WebSocket** via `react-use-websocket`
- **Vite** (rolldown variant) for bundling

## Code Style

### Rust Modules

- Use `ansi_term::Color` for colorized logging
- Log format: `[MODULE_NAME] [LEVEL] [THREAD id] [FILE line] message`
- All async operations use Tokio
- Wrap shared state in `AsyncModifiable<T>` (type alias for `Arc<Mutex<T>>`)
- MySQL queries use `mysql_async` with `.ignore()` for DDL statements

### Frontend

- Components in `client_web/src/router/`
- Use TypeScript with strict mode
- WebSocket messages follow `{command: string, content: any}` structure
- Responsive design with Fluent UI theming

## Legacy Python Code

The repository contains legacy Python modules in the root directory (`ws_server/`, `judge/`, `chat_server/`, etc.) that are no longer used. The active backend is entirely in `rust_backend/`. Do not modify Python backend code unless specifically working on migration artifacts.
