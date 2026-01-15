#!/bin/bash
source ~/.bashrc
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"
cd "${PROJECT_DIR}" || exit 1

GIT_DESCRIBE=$(git describe --dirty --tags)
docker buildx build --ssh default --tag dx-rpg:latest --build-arg GIT_DESCRIBE="$GIT_DESCRIBE" .
