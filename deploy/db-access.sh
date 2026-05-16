#!/bin/bash
# Access sqlite-web in a browser.
#
# On Linux / macOS / VPS:
#   Port 8082 works directly → open http://localhost:8082
#
# On Windows with Rancher Desktop:
#   Rancher Desktop's host-switch has a known routing bug when multiple
#   containers in the same network expose the same internal port (8080).
#   Workaround: access sqlite-web via its Docker-internal IP using WSL2.
#
set -e
cd "$(dirname "$0")"

CONTAINER="dx-rpg-sqlite-web-1"

# Check the container is running
if ! docker inspect "$CONTAINER" &>/dev/null; then
  echo "sqlite-web container not running. Start with: ./start.sh"
  exit 1
fi

SQLITE_IP=$(docker inspect "$CONTAINER" --format '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}')
SQLITE_PORT=5000

echo "sqlite-web is running at:"
echo ""
echo "  Internal Docker IP : http://${SQLITE_IP}:${SQLITE_PORT}"
echo "  Host port (Linux)  : http://localhost:8082"
echo ""

# Detect OS
case "$(uname -s)" in
  Linux*)
    echo "Opening http://localhost:8082 ..."
    xdg-open "http://localhost:8082" 2>/dev/null || echo "Open http://localhost:8082 in your browser."
    ;;
  Darwin*)
    echo "Opening http://localhost:8082 ..."
    open "http://localhost:8082"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    # Windows Git Bash / Rancher Desktop
    echo "Rancher Desktop detected."
    echo ""
    echo "Option 1 — Try the WSL2 container IP directly (works in most browsers):"
    echo "  http://${SQLITE_IP}:${SQLITE_PORT}"
    echo ""
    echo "Option 2 — Quick CLI dump of the database:"
    echo "  docker exec ${CONTAINER} sqlite3 //data/db.sqlite .tables"
    echo "  docker exec ${CONTAINER} sqlite3 //data/db.sqlite 'SELECT * FROM users;'"
    echo ""
    echo "Option 3 — Use Rancher Desktop UI: Containers → ${CONTAINER} → Port Forwarding"
    echo ""
    # Try to open the WSL2 IP in the default Windows browser
    if command -v cmd.exe &>/dev/null; then
      cmd.exe /c start "http://${SQLITE_IP}:${SQLITE_PORT}" 2>/dev/null || true
    fi
    ;;
  *)
    echo "Open http://localhost:8082 (Linux/VPS) or http://${SQLITE_IP}:${SQLITE_PORT} (Docker internal)."
    ;;
esac
