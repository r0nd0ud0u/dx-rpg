#!/bin/bash
dx bundle --platform web --release --out_dir bundle
cp -r offlines bundle/offlines/
mkdir -p bundle/output/games