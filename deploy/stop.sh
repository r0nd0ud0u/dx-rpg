#!/bin/bash
# Stop the stack. Volumes (db + saves) are preserved — data is NOT lost.
# To fully reset data: docker compose down -v
set -e
cd "$(dirname "$0")"

docker compose down
echo "Stack stopped. Volumes preserved. Run ./start.sh to restart."
