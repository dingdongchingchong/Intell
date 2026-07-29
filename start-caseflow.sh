#!/usr/bin/env bash
# Start CaseFlow CMS backend (:8080) and frontend (:3000) together.
# Usage: ./start-caseflow.sh
# Stop:  Ctrl+C

set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
BACKEND_DIR="$ROOT/backend"
FRONTEND_DIR="$ROOT/frontend"
LOG_DIR="${CASEFLOW_LOG_DIR:-$ROOT/logs}"
BACKEND_PORT="${BACKEND_PORT:-8080}"
FRONTEND_PORT="${FRONTEND_PORT:-3000}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$BACKEND_DIR/target}"

mkdir -p "$LOG_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

BACKEND_PID=""
FRONTEND_PID=""

cleanup() {
  echo ""
  echo -e "${YELLOW}Stopping CaseFlow…${NC}"
  if [[ -n "$FRONTEND_PID" ]] && kill -0 "$FRONTEND_PID" 2>/dev/null; then
    kill "$FRONTEND_PID" 2>/dev/null || true
    wait "$FRONTEND_PID" 2>/dev/null || true
  fi
  if [[ -n "$BACKEND_PID" ]] && kill -0 "$BACKEND_PID" 2>/dev/null; then
    kill "$BACKEND_PID" 2>/dev/null || true
    wait "$BACKEND_PID" 2>/dev/null || true
  fi
  echo -e "${GREEN}Stopped.${NC}"
}
trap cleanup EXIT INT TERM

port_busy() {
  local port="$1"
  ss -lnt 2>/dev/null | grep -q ":${port} " || netstat -lnt 2>/dev/null | grep -q ":${port} "
}

ensure_postgres() {
  if command -v docker >/dev/null 2>&1; then
    if docker ps -a --format '{{.Names}}' 2>/dev/null | grep -qx 'caseflow-postgres'; then
      if ! docker ps --format '{{.Names}}' 2>/dev/null | grep -qx 'caseflow-postgres'; then
        echo -e "${YELLOW}Starting PostgreSQL (caseflow-postgres)…${NC}"
        docker start caseflow-postgres >/dev/null
        sleep 2
      fi
      return 0
    fi
  fi
  # Best-effort: DB may already be local
  return 0
}

if port_busy "$BACKEND_PORT"; then
  echo -e "${RED}Port ${BACKEND_PORT} is already in use.${NC}"
  echo "Stop the other process or: fuser -k ${BACKEND_PORT}/tcp"
  exit 1
fi
if port_busy "$FRONTEND_PORT"; then
  echo -e "${RED}Port ${FRONTEND_PORT} is already in use.${NC}"
  echo "Stop the other process or: fuser -k ${FRONTEND_PORT}/tcp"
  exit 1
fi

ensure_postgres

echo -e "${GREEN}CaseFlow CMS${NC}"
echo "=============="
echo "Backend  → http://127.0.0.1:${BACKEND_PORT}  (log: $LOG_DIR/backend.log)"
echo "Frontend → http://127.0.0.1:${FRONTEND_PORT}  (log: $LOG_DIR/frontend.log)"
echo "Tailscale: http://100.68.147.74:${FRONTEND_PORT}"
echo ""
echo -e "${YELLOW}Ctrl+C stops both.${NC}"
echo ""

(
  cd "$BACKEND_DIR"
  export CARGO_TARGET_DIR
  exec cargo run
) >"$LOG_DIR/backend.log" 2>&1 &
BACKEND_PID=$!

(
  cd "$FRONTEND_DIR"
  exec python3 -m http.server "$FRONTEND_PORT" --bind 0.0.0.0
) >"$LOG_DIR/frontend.log" 2>&1 &
FRONTEND_PID=$!

# Wait until backend answers /health (or fails)
echo -n "Waiting for API"
for _ in $(seq 1 90); do
  if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
    echo ""
    echo -e "${RED}Backend exited early. Last log lines:${NC}"
    tail -n 40 "$LOG_DIR/backend.log" || true
    exit 1
  fi
  if curl -fsS -m 1 "http://127.0.0.1:${BACKEND_PORT}/health" >/dev/null 2>&1; then
    echo -e " ${GREEN}ready${NC}"
    break
  fi
  echo -n "."
  sleep 1
done

if ! curl -fsS -m 2 "http://127.0.0.1:${BACKEND_PORT}/health" >/dev/null 2>&1; then
  echo ""
  echo -e "${YELLOW}API not healthy yet — check $LOG_DIR/backend.log${NC}"
fi

if curl -fsS -m 2 -o /dev/null "http://127.0.0.1:${FRONTEND_PORT}/"; then
  echo -e "UI ${GREEN}ready${NC} → http://127.0.0.1:${FRONTEND_PORT}/"
else
  echo -e "${YELLOW}UI not responding yet — check $LOG_DIR/frontend.log${NC}"
fi

echo ""
# Keep script alive while children run
wait
