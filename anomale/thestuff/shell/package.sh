#!/bin/bash
set -e

# Configuration
PROJECT_NAME="anomale"
DIST_DIR="dist"
SOURCE_TAR="${DIST_DIR}/${PROJECT_NAME}-source.tar.gz"
BIN_TAR="${DIST_DIR}/${PROJECT_NAME}-bin.tar.gz"

# clean up previous dist
echo "Cleaning up previous builds..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# verify we are in the project root
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# 1. Source Package
echo "Creating source package..."
# Exclude target/, dist/, .git/ (if it existed), and other temp files.
tar --exclude='./target' \
    --exclude="./$DIST_DIR" \
    --exclude='./.git' \
    --exclude='./.idea' \
    --exclude='./.vscode' \
    -czvf "$SOURCE_TAR" .

echo "Source package created at: $SOURCE_TAR"

# 2. Binary Package
echo "Building release binary..."
cargo build --release

echo "Creating binary package..."
# Create a temporary directory for buffering the binary package content
BIN_TMP_DIR="${DIST_DIR}/bin_tmp"
mkdir -p "$BIN_TMP_DIR"

# Copy binary and config
cp "target/release/$PROJECT_NAME" "$BIN_TMP_DIR/"
if [ -f "config.conf" ]; then
    cp "config.conf" "$BIN_TMP_DIR/"
else
    echo "Warning: config.conf not found, skipping."
fi

# compress
tar -C "$BIN_TMP_DIR" -czvf "$BIN_TAR" .

# cleanup tmp
rm -rf "$BIN_TMP_DIR"

echo "Binary package created at: $BIN_TAR"
echo "Done!"
