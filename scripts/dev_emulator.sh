#!/bin/bash
# Boots an Android emulator for local `dx serve --platform android` testing.
# Usage: ./scripts/dev_emulator.sh [avd-name]
# List available AVDs with: emulator -list-avds
AVD="${1:?Usage: $0 <avd-name>  (list available AVDs with: emulator -list-avds)}"
emulator -avd "$AVD"
