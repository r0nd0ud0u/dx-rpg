#!/bin/bash
# Start the full stack (app + sqlite-web) with Docker Compose.
# Data is persisted in named Docker volumes across stop/start cycles.
#
# On first run, Docker will pull the latest image from ghcr.io.
# To use a locally built image instead, run docker_build.sh first and
# change the image tag in docker-compose.yml to dx-rpg:latest.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Determine the volume name (project name + _db_data)
PROJECT_NAME="$(basename "$PWD")"
VOLUME_NAME="${PROJECT_NAME}_db_data"

# Stop any running containers first
echo "Stopping any existing containers..."
docker compose down 2>/dev/null || true

# Ensure /data/db.sqlite exists inside the volume before starting the app.
# SQLite can create the file but only if the parent directory is writable.
# This step is a safety net for first-run or empty-volume situations.
echo "Ensuring database file exists in volume '$VOLUME_NAME'..."
docker run --rm \
  -v "${VOLUME_NAME}:/data" \
  alpine sh -c "mkdir -p /data && touch /data/db.sqlite && chmod 666 /data/db.sqlite"
echo "  /data/db.sqlite is ready."

echo "Starting dx-rpg stack..."
docker compose up -d

echo ""
echo "Services:"
echo "  App        → http://localhost:8080"
echo "  SQLite UI  → http://localhost:8082  (loopback only — SSH tunnel for remote access)"
echo ""
echo "To follow logs:  ./scripts/logs.sh"
echo "To stop:         ./scripts/docker_compose_down.sh"
