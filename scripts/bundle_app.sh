#!/bin/bash
dx bundle --platform web --release --out-dir bundle
cp -r offlines bundle/offlines/
mkdir -p bundle/output/games
cp .env_template bundle/.env_template
cp db.sqlite bundle/db.sqlite