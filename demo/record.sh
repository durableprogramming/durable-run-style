#!/bin/bash

# Script to record the demo using VHS

# Check if VHS is installed
if ! command -v vhs &> /dev/null; then
    echo "VHS is not installed. Please install VHS first."
    echo "Visit: https://github.com/charmbracelet/vhs"
    exit 1
fi

# Change to the project root directory
cd "$(dirname "$0")/.."

docker compose down

# Run the VHS tape to record the demo
vhs demo/demo.tape
