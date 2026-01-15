#!/bin/bash
dx bundle --platform web --release
cp -r offlines bundle/offlines/
mkdir -p bundle/output/games