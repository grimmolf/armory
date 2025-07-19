#!/bin/bash
# Docker-based cross-compilation script for Armory Rust

set -e

TARGET=${1:-x86_64-unknown-linux-gnu}
OUTPUT_DIR="/workspace/output"

echo "🐳 Building Armory Rust for target: $TARGET"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build the binary
cd /workspace/armory-rust

echo "📦 Running cargo build for $TARGET..."
cargo build --release --target "$TARGET"

# Copy binary to output
BINARY_NAME="armory-rust"
cp "target/$TARGET/release/$BINARY_NAME" "$OUTPUT_DIR/$BINARY_NAME-$TARGET"

echo "✅ Build completed: $OUTPUT_DIR/$BINARY_NAME-$TARGET"

# Show binary info
echo "📊 Binary information:"
file "$OUTPUT_DIR/$BINARY_NAME-$TARGET"
ls -lh "$OUTPUT_DIR/$BINARY_NAME-$TARGET"