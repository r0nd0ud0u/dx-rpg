#!/bin/bash
dx bundle --platform web --release --out-dir dx-rpg_windows --package-types "exe"
cp -r offlines dx-rpg_windows/offlines/
mkdir -p dx-rpg_windows/output/games
cp .env dx-rpg_windows/.env