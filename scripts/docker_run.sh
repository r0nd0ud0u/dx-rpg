#!/bin/bash
docker run --rm --user="$(id --user):$(id --group)" -p 8080:8080 dx-rpg -vv
