#!/bin/sh

# Build the tool in release mode
cargo build -r || exit 1

# Locate the crates source directory in ~/.cargo/registry/src/
# We look for the index.crates.io-* directory
CRATES_DIR=$(ls -d "${HOME}/.cargo/registry/src/index.crates.io-"* 2>/dev/null | head -n 1)

if [ -z "$CRATES_DIR" ]; then
    echo "Error: Could not find Cargo registry source directory in ~/.cargo/registry/src/" >&2
    exit 1
fi

echo "Using crates directory: $CRATES_DIR"

# Generate the license contents for all dependencies
# We use the release binary we just built
./target/release/cratelist Cargo.lock --license-contents "$CRATES_DIR" > DEPENDENCIES_LICENSE

echo "License contents for all dependencies have been compiled into DEPENDENCIES_LICENSE"
echo "You can now commit this file to the repository."
