#!/bin/bash
set -e

# Build the project
cargo build --release

# Ensure ~/.local/bin exists
mkdir -p ~/.local/bin

# Copy the binary and rename to druns
cp target/release/druns ~/.local/bin/druns

echo "Installed druns to ~/.local/bin/druns"
