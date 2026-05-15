#!/bin/bash
# Build the Docker image locally (uses SSH agent for private cargo deps)
eval $(ssh-agent)
ssh-add
GIT_DESCRIBE=$(git describe --dirty --tags 2>/dev/null || echo "dev")
docker buildx build \
  --ssh default \
  --tag dx-rpg:latest \
  --build-arg GIT_DESCRIBE="$GIT_DESCRIBE" \
  .
