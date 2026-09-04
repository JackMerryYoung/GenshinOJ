# Repository Guide

## Project

RsOJ is an online judge with a React/TypeScript frontend and a modular Rust backend. The Python
services under the repository root are migration artifacts; active backend work belongs under
`rust_backend/`.

## Start And Verify

Use the root launcher for local development:

```bash
./start.sh
./start.sh --release
./start.sh --skip-build
```

It builds the Rust workspace, deploys module `.so` files to `rust_backend/modules/<id>/`, then
starts `main_backend` and Vite. The backend must run with `rust_backend/` as its working directory
because configuration and problem paths are resolved from it.

Verification commands:

```bash
cd rust_backend
cargo check --workspace
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings

cd ../client_web
npm run lint
npm run build
npm run test:e2e
```

## Backend Architecture

`main_backend` loads dynamic libraries according to `module_config_rs.json`. Enabled modules export:

```rust
on_init(...) -> (tokio::runtime::Runtime, AsyncModifiable<ModuleStatus>)
on_unload(unload_timeout_ms: usize)
```

Each module owns a Tokio runtime. Dependencies load in topological order; shutdown cleanup runs
while all runtimes are alive, then runtimes stop in reverse order. In restricted mode, a reported
module panic shuts down the process.

Current modules:

- `main_backend`: loader and lifecycle coordinator
- `ws_server`: browser WebSocket and avatar HTTP routes, normally port 9983
- `simple_authenticator`: login, session restore, session validation and connection identity
- `judge`: problems, submissions, solutions, discussions and notifications
- `chat_server`: direct messages and chat history
- `userish`: profiles, search, follows and friends
- `control_panel`: local Axum administration API, normally port 9990
- `db_connector`: disabled compatibility module

## WebSocket Protocol

The browser sends an envelope using `type`, not `command`:

```json
{
  "type": "problem_set",
  "content": { "request_key": "client-generated-id" }
}
```

`ws_server` forwards it internally as `on_<type>` with a server-assigned `ws_id`. Modules bind
handlers through `rust_backend/ws_server/ws_server_config_rs.json`. Request/response flows must
preserve the browser's `content.request_key`; the frontend `WebSocketRequestBroker` uses it to pair
concurrent responses.

To add a command:

1. Add `on_<type>` to `ws_server_config_rs.json`.
2. Implement and export the handler in the owning module.
3. Include the command in that module's bind/unbind lists.
4. Use the frontend request broker for request/response operations.
5. Add protocol and Playwright coverage where practical.

## Database And Judge

Modules currently connect to `mysql://root:123456@127.0.0.1:3306/`; this is hard-coded and is not
overridden by `DB_*` environment variables. Tables and additive migrations run during module
initialization.

The judge reads testcase files from `problem/<number>/` and runs `gcc`, `g++`, `javac`, `java`, or
`python3`. It enforces wall-clock, virtual-memory and output limits, but does not provide a security
sandbox. Do not treat it as safe for arbitrary untrusted code.

## Frontend

The frontend uses React 19, TypeScript, Fluent UI, Redux Toolkit, React Router, i18next and
`react-use-websocket`. Vite proxies `/wsapi` and `/avatar` to the WebSocket service. Control-panel
requests use `src/controlPanelApi.ts` and `VITE_CONTROL_PANEL_URL`.

End-to-end tests live in `client_web/e2e/`. They mock WebSocket and control-panel HTTP traffic so
they can run without MySQL or the Rust backend.

## Control Panel

There is no debug authentication bypass and no browser token-generation flow. Role tokens come
from `CONTROL_PANEL_ADMIN_TOKEN`, `CONTROL_PANEL_PROBLEM_ADMIN_TOKEN`, and
`CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN`. See `CONTROL_PANEL.md` before changing routes or roles;
authorization must be enforced in the backend middleware, not only in React.
