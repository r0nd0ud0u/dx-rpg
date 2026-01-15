#!/bin/bash
eval $(ssh-agent)
ssh-add
GIT_DESCRIBE=$(git describe --dirty --tags)
docker buildx build --ssh default --tag dx-rpg:latest --build-arg GIT_DESCRIBE="$GIT_DESCRIBE" .
