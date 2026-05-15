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

echo "Starting dx-rpg stack..."
docker compose up -d

echo ""
echo "Services:"
echo "  App        → http://localhost:8080"
echo "  SQLite UI  → http://localhost:8082  (loopback only — SSH tunnel for remote access)"
echo ""
echo "To follow logs:  docker compose logs -f"
echo "To stop:         ./scripts/docker_compose_down.sh"
