#!/bin/bash

# Get version from Cargo.toml
VERSION=$(grep '^version =' Cargo.toml | head -1 | cut -d '"' -f 2)

# Get optional name/suffix
SUFFIX=$1

# Build the project in release mode
echo "Building version $VERSION..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# Ensure versions directory exists
mkdir -p versions

# Construct filename
if [ -z "$SUFFIX" ]; then
    FILENAME="chess_v$VERSION"
else
    FILENAME="chess_v${VERSION}_$SUFFIX"
fi

# Copy binary to versions directory
cp target/release/chess "versions/$FILENAME"

echo "Engine version $VERSION saved to versions/$FILENAME"
ls -l "versions/$FILENAME"
