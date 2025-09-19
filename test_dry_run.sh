#!/bin/bash

# Simple test script for dry-run functionality
cd "$(dirname "$0")"

# Create a simple test script
mkdir -p test_scripts
cat > test_scripts/script.yaml << 'EOF'
tasks:
  hello:
    command: echo "Hello, world!"
    description: "Simple hello world task"
  build:
    command: cargo build
    description: "Build the project"
    env:
      RUST_LOG: debug
EOF

echo "Testing dry-run functionality..."

echo ""
echo "=== Dry-Run Mode ==="
# Test dry-run mode
BODO_ROOT_SCRIPT=test_scripts/script.yaml cargo run -- --dry-run hello

echo ""
echo "=== Normal Mode ==="
# Test normal mode (commented out to avoid actual execution)
# BODO_ROOT_SCRIPT=test_scripts/script.yaml cargo run -- hello

# Clean up
rm -rf test_scripts