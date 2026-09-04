#!/usr/bin/env bash

set -Eeuo pipefail

PROJECT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="${PROJECT_DIR}/rust_backend"
FRONTEND_DIR="${PROJECT_DIR}/client_web"

BUILD_PROFILE="debug"
SKIP_BUILD=0
FRONTEND_HOST="${FRONTEND_HOST:-127.0.0.1}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"

usage() {
    cat <<'EOF'
Usage: ./start.sh [options]

Build and start the Rust backend and Vite frontend together.

Options:
  --release          Build and run the release backend.
  --skip-build       Reuse the currently deployed backend binaries.
  --host HOST        Vite listen host (default: 127.0.0.1).
  --port PORT        Vite listen port (default: 5173).
  -h, --help         Show this help message.

Environment:
  FRONTEND_HOST
  FRONTEND_PORT
  CONTROL_PANEL_ADMIN_TOKEN
  CONTROL_PANEL_PROBLEM_ADMIN_TOKEN
  CONTROL_PANEL_COMMUNITY_ADMIN_TOKEN
  CONTROL_PANEL_CORS_ORIGIN
  RSOJ_JUDGE_CONCURRENCY
EOF
}

fail() {
    printf 'start.sh: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

while (($# > 0)); do
    case "$1" in
        --release)
            BUILD_PROFILE="release"
            ;;
        --skip-build)
            SKIP_BUILD=1
            ;;
        --host)
            (($# >= 2)) || fail "--host requires a value"
            FRONTEND_HOST="$2"
            shift
            ;;
        --port)
            (($# >= 2)) || fail "--port requires a value"
            FRONTEND_PORT="$2"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            fail "unknown option: $1"
            ;;
    esac
    shift
done

[[ "${FRONTEND_PORT}" =~ ^[0-9]+$ ]] || fail "frontend port must be numeric"
((FRONTEND_PORT >= 1 && FRONTEND_PORT <= 65535)) || fail "frontend port must be between 1 and 65535"
[[ "$(uname -s)" == "Linux" ]] || fail "the Rust module loader currently expects Linux .so files"

require_command cargo
require_command npm
require_command install

VITE_BIN="${FRONTEND_DIR}/node_modules/.bin/vite"
[[ -x "${VITE_BIN}" ]] || fail "frontend dependencies are missing; run 'cd client_web && npm install' first"

MODULE_PACKAGES=(
    ws_server
    simple_authenticator
    db_connector
    chat_server
    judge
    userish
    control_panel
)

deploy_backend() {
    local cargo_args=(build --workspace)
    if [[ "${BUILD_PROFILE}" == "release" ]]; then
        cargo_args+=(--release)
    fi

    printf '[launcher] Building Rust workspace (%s)...\n' "${BUILD_PROFILE}"
    (cd "${BACKEND_DIR}" && cargo "${cargo_args[@]}")

    local target_dir="${BACKEND_DIR}/target/${BUILD_PROFILE}"
    local module source destination temporary
    for module in "${MODULE_PACKAGES[@]}"; do
        source="${target_dir}/lib${module}.so"
        destination="${BACKEND_DIR}/modules/${module}/lib${module}.so"
        temporary="${destination}.tmp.$$"
        [[ -f "${source}" ]] || fail "Cargo did not produce ${source}"
        mkdir -p -- "$(dirname -- "${destination}")"
        install -m 755 -- "${source}" "${temporary}"
        mv -f -- "${temporary}" "${destination}"
    done
}

BACKEND_BIN="${BACKEND_DIR}/target/${BUILD_PROFILE}/main_backend"

if ((SKIP_BUILD == 0)); then
    deploy_backend
else
    [[ -x "${BACKEND_BIN}" ]] || fail "backend binary is missing: ${BACKEND_BIN}"
    for module in "${MODULE_PACKAGES[@]}"; do
        [[ -f "${BACKEND_DIR}/modules/${module}/lib${module}.so" ]] ||
            fail "deployed module is missing: ${module}"
    done
fi

backend_pid=""
frontend_pid=""
stopping=0

stop_processes() {
    if ((stopping == 1)); then
        return
    fi
    stopping=1
    trap - INT TERM

    if [[ -n "${backend_pid}" ]] && kill -0 "${backend_pid}" 2>/dev/null; then
        printf '\n[launcher] Stopping backend gracefully...\n'
        kill -INT "${backend_pid}" 2>/dev/null || true
    fi
    if [[ -n "${frontend_pid}" ]] && kill -0 "${frontend_pid}" 2>/dev/null; then
        printf '[launcher] Stopping frontend...\n'
        kill -TERM "${frontend_pid}" 2>/dev/null || true
    fi

    if [[ -n "${backend_pid}" ]]; then
        wait "${backend_pid}" 2>/dev/null || true
    fi
    if [[ -n "${frontend_pid}" ]]; then
        wait "${frontend_pid}" 2>/dev/null || true
    fi
}

handle_signal() {
    stop_processes
    exit 130
}

trap stop_processes EXIT
trap handle_signal INT TERM

(cd "${BACKEND_DIR}" && exec "${BACKEND_BIN}") &
backend_pid=$!

(cd "${FRONTEND_DIR}" && exec "${VITE_BIN}" \
    --host "${FRONTEND_HOST}" \
    --port "${FRONTEND_PORT}" \
    --strictPort) &
frontend_pid=$!

printf '[launcher] Backend PID: %s\n' "${backend_pid}"
printf '[launcher] Frontend: http://%s:%s (PID %s)\n' "${FRONTEND_HOST}" "${FRONTEND_PORT}" "${frontend_pid}"
printf '[launcher] Press Ctrl+C to stop both processes.\n'

if wait -n "${backend_pid}" "${frontend_pid}"; then
    child_status=0
else
    child_status=$?
fi

if ! kill -0 "${backend_pid}" 2>/dev/null; then
    printf '[launcher] Backend exited; shutting down the frontend.\n' >&2
else
    printf '[launcher] Frontend exited; shutting down the backend.\n' >&2
fi

exit "${child_status}"
