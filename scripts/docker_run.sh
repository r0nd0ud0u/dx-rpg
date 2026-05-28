#!/bin/bash
# Run a single container (no compose, no persistence). For development/testing only.
# For production with persistent data, use docker_compose_up.sh instead.
docker run --rm \
  -p 8081:8080 \
  -e DATABASE_URL="sqlite:///data/db.sqlite" \
  -e PORT=8080 \
  -e IP=0.0.0.0 \
  -v dx_rpg_db_data:/data \
  -v dx_rpg_saved_data:/usr/local/app/saved_data \
  -v dx_rpg_photos_data:/usr/local/app/photos \
  dx-rpg:latest
