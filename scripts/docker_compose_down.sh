#!/bin/bash
# Stop the Docker Compose stack.
# Volumes (db + saved_data) are preserved — data is NOT lost.
#
# To also remove volumes (full reset): docker compose down -v

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "Stopping dx-rpg stack..."
docker compose down
echo "Done. Volumes are preserved (data not lost)."
echo "To fully reset volumes: docker compose down -v"
