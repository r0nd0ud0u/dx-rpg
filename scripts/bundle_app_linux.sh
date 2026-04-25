#!/bin/bash
dx bundle --platform web --release --out-dir dx-rpg_linux --package-types "deb"
cp -r offlines dx-rpg_linux/offlines/
mkdir -p dx-rpg_linux/output/games
cp .env dx-rpg_linux/.env