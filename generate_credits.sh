#!/bin/sh

# Build the tool in release mode
cargo build -r || exit 1

# Vendor all dependencies into ./vendor.
#
# We deliberately do NOT read ~/.cargo/registry/src/, because `cargo build`
# only extracts sources for the host target. Platform-specific dependencies
# (e.g. Windows-only crates) are listed in Cargo.lock but never appear there,
# which previously produced bogus "No license file found" entries.
#
# `cargo vendor` materialises the full source (Cargo.toml + license files) of
# *every* crate in Cargo.lock, for all platforms, into a single directory.
# --versioned-dirs forces "<name>-<version>" naming, which matches the layout
# cratelist expects.
VENDOR_DIR="vendor"
echo "Vendoring all dependencies into ${VENDOR_DIR}/ ..."
cargo vendor --versioned-dirs --locked "$VENDOR_DIR" >/dev/null || exit 1

# Generate the license contents for all dependencies
# We use the release binary we just built
./target/release/cratelist Cargo.lock --license-contents "$VENDOR_DIR" > DEPENDENCIES_LICENSE

# A few crates legitimately ship without standard license files; those show up
# as "No license file found". This is informational, not an error.
if grep -q "No license file found" DEPENDENCIES_LICENSE; then
    echo "Note: some dependencies lack standard license files:" >&2
    grep "No license file found" DEPENDENCIES_LICENSE >&2
fi

# Compress the license file to reduce binary size when embedded
# We keep both the plain text and compressed files
gzip -fk DEPENDENCIES_LICENSE

echo "License contents for all dependencies have been compiled into DEPENDENCIES_LICENSE and compressed into DEPENDENCIES_LICENSE.gz"
echo "You can now commit these files to the repository."
