#!/usr/bin/env bash
set -euo pipefail

# Deploy script for the Rust backend, API Playground, and Svelte resume editor.
#
# Usage:
#   ./deploy.sh --ssh user@host --api-url https://server.site.dev/api
#
# See --help for all options.

REMOTE_BASE="${REMOTE_BASE:-$HOME/public_html}"
RESTART_CMD="${RESTART_CMD:-restart}"

SKIP_BACKEND=false
SKIP_PLAYGROUND=false
SKIP_RESUME=false
SKIP_RSYNC=false
SKIP_RESTART=false

usage() {
    cat <<EOF
Usage: $0 --ssh <user@host> --api-url <https://.../api> [options]

Deploy the backend binary, API Playground, and Svelte resume editor to a
remote server.

Required:
  --ssh DEST          SSH destination (e.g. user@192.168.1.10)
  --api-url URL       API base URL passed to the resume editor build

Remote paths:
  --remote-base DIR              Base directory on the server
                                   (default: ~/public_html)
  --remote-node-build DIR        Destination for the Svelte build
                                   (default: <remote-base>/node_build)
  --remote-playground-dist DIR   Destination for the API Playground dist
                                   (default: <remote-base>/playground_dist)
  --remote-binary PATH           Destination for the backend binary
                                   (default: <remote-base>/target/release/projects_backend_database)
  --restart-cmd CMD              Command to run on the server to restart
                                   (default: restart)

Skip flags:
  --skip-backend      Skip Docker cross-compile of the Rust backend
  --skip-playground   Skip API Playground WASM build
  --skip-resume       Skip Svelte resume editor build
  --skip-rsync        Skip rsyncing artifacts to the server
  --skip-restart      Skip running the restart command on the server

Other:
  -h, --help          Show this help

Examples:
  $0 --ssh admin@192.168.1.10 --api-url https://server.site.dev/api
  $0 --ssh admin@server --api-url https://api.example.com/api --skip-restart
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --ssh)
            SSH_DEST="$2"
            shift 2
            ;;
        --api-url)
            API_URL="$2"
            shift 2
            ;;
        --remote-base)
            REMOTE_BASE="$2"
            shift 2
            ;;
        --remote-node-build)
            REMOTE_NODE_BUILD="$2"
            shift 2
            ;;
        --remote-playground-dist)
            REMOTE_PLAYGROUND_DIST="$2"
            shift 2
            ;;
        --remote-binary)
            REMOTE_BINARY="$2"
            shift 2
            ;;
        --restart-cmd)
            RESTART_CMD="$2"
            shift 2
            ;;
        --skip-backend)
            SKIP_BACKEND=true
            shift
            ;;
        --skip-playground)
            SKIP_PLAYGROUND=true
            shift
            ;;
        --skip-resume)
            SKIP_RESUME=true
            shift
            ;;
        --skip-rsync)
            SKIP_RSYNC=true
            shift
            ;;
        --skip-restart)
            SKIP_RESTART=true
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

# Validate required arguments.
if [[ -z "${SSH_DEST:-}" || -z "${API_URL:-}" ]]; then
    echo "Error: --ssh and --api-url are required." >&2
    usage >&2
    exit 1
fi

# Resolve remote paths with sensible defaults.
REMOTE_NODE_BUILD="${REMOTE_NODE_BUILD:-$REMOTE_BASE/node_build}"
REMOTE_PLAYGROUND_DIST="${REMOTE_PLAYGROUND_DIST:-$REMOTE_BASE/playground_dist}"
REMOTE_BINARY="${REMOTE_BINARY:-$REMOTE_BASE/target/release/projects_backend_database}"
REMOTE_BINARY_DIR=$(dirname "$REMOTE_BINARY")

# Ensure the script is run from the backend repository root.
if [[ ! -f "Containerfile" || ! -d "api_playground" ]]; then
    echo "Error: Please run this script from the projects_backend_database repository root." >&2
    exit 1
fi

BACKEND_DIR="$(pwd)"
RESUME_EDITOR_DIR="$(cd "$BACKEND_DIR/../resume_editor_svelte" && pwd)"

if [[ ! -d "$RESUME_EDITOR_DIR" ]]; then
    echo "Error: resume_editor_svelte directory not found at $BACKEND_DIR/../resume_editor_svelte" >&2
    exit 1
fi

log() {
    echo "[deploy] $*"
}

run_ssh() {
    ssh "$SSH_DEST" "$@"
}

# ---------------------------------------------------------------------------
# Build steps
# ---------------------------------------------------------------------------

if [[ "$SKIP_BACKEND" == false ]]; then
    log "Cross-compiling Rust backend for linux/arm64..."
    docker buildx build \
        --platform linux/arm64 \
        --target artifact \
        --output type=local,dest=./docker-build-out \
        -f Containerfile .
    log "Backend binary built: ./docker-build-out/out/projects_backend_database"
else
    log "Skipping backend build."
fi

if [[ "$SKIP_PLAYGROUND" == false ]]; then
    log "Building API Playground WASM frontend..."
    (cd api_playground && ./build.sh)
    log "API Playground built: ./api_playground/dist/"
else
    log "Skipping API Playground build."
fi

if [[ "$SKIP_RESUME" == false ]]; then
    log "Building Svelte resume editor with API_URL=$API_URL..."
    (cd "$RESUME_EDITOR_DIR" && VITE_PUBLIC_API_BASE_URL="$API_URL" npm run build)
    log "Svelte resume editor built: $RESUME_EDITOR_DIR/build/"
else
    log "Skipping Svelte resume editor build."
fi

# ---------------------------------------------------------------------------
# Rsync steps
# ---------------------------------------------------------------------------

if [[ "$SKIP_RSYNC" == false ]]; then
    log "Ensuring remote directories exist on $SSH_DEST..."
    run_ssh "mkdir -p \"$REMOTE_NODE_BUILD\" \"$REMOTE_PLAYGROUND_DIST\" \"$REMOTE_BINARY_DIR\""

    log "Rsyncing Svelte resume editor to $SSH_DEST:$REMOTE_NODE_BUILD/"
    rsync -avz --delete "$RESUME_EDITOR_DIR/build/" "$SSH_DEST:$REMOTE_NODE_BUILD/"

    log "Rsyncing API Playground to $SSH_DEST:$REMOTE_PLAYGROUND_DIST/"
    rsync -avz --delete "$BACKEND_DIR/api_playground/dist/" "$SSH_DEST:$REMOTE_PLAYGROUND_DIST/"

    log "Rsyncing backend binary to $SSH_DEST:$REMOTE_BINARY"
    rsync -avz "$BACKEND_DIR/docker-build-out/out/projects_backend_database" "$SSH_DEST:$REMOTE_BINARY"
else
    log "Skipping rsync."
fi

# ---------------------------------------------------------------------------
# Restart step
# ---------------------------------------------------------------------------

if [[ "$SKIP_RESTART" == false ]]; then
    log "Running restart command on $SSH_DEST: $RESTART_CMD"
    run_ssh "chmod +x \"$REMOTE_BINARY\" && $RESTART_CMD"
    log "Restart command executed."
else
    log "Skipping restart."
fi

log "Deployment complete."
