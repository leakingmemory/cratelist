# cratelist

`cratelist` is a command-line utility for working with Rust projects. It helps you list dependencies from `Cargo.lock`, manage local crate caches, extract license information, and generate Flatpak source lists.

## Features

- **Dependency Listing**: Easily list all dependencies and their versions from your `Cargo.lock` file.
- **Crate Cache Management**: Clean up local crate caches or vendor directories based on the dependencies of your project.
- **License Extraction**: Extract and view license information for all your project's dependencies directly from the crate sources.
- **Flatpak Integration**: Generate `cargo-sources.json` to facilitate building Rust applications as Flatpaks.
- **Embedded Licenses**: View both the project's license and all dependency licenses that have been collected.

## Installation

### From Source

You can build and install `cratelist` using Cargo:

```bash
cargo install --path .
```

Alternatively, you can build the release binary and use the provided installation script:

```bash
cargo build --release
sudo ./install.sh
```

### Snap

`cratelist` is available as a snap. You can install it using:

```bash
sudo snap install cratelist
```

*Note: You may need to connect interfaces for home and removable-media access if you intend to use it outside your home directory.*

### Flatpak

`cratelist` can be built and installed as a Flatpak using the provided manifest:

```bash
flatpak-builder --user --install --force-clean build-dir flatpak/flatpak.json
```

## Usage

Basic usage requires providing the path to a `Cargo.lock` file:

```bash
cratelist /path/to/Cargo.lock
```

### Options

- `-d, --delete <DIR>`: Delete all listed crates from the specified directory (useful for cleaning up `~/.cargo/registry/src`).
- `-l, --licenses <DIR>`: List the licenses of all dependencies (requires providing the directory where crates are stored, e.g., `~/.cargo/registry/src/index.crates.io-...`).
- `-C, --license-contents <DIR>`: Show the full contents of license files for all dependencies.
- `-L, --show-embedded-licenses`: Print the embedded licenses of the project and its dependencies.
- `-D, --dash`: Use `-` (dash) as a version separator instead of `@` when listing dependencies.
- `-F, --flatpak`: Generate Flatpak `cargo-sources.json` format to stdout.

### Examples

**List dependencies with version separated by @:**
```bash
cratelist Cargo.lock
```

**Generate Flatpak sources:**
```bash
cratelist Cargo.lock --flatpak > flatpak/cargo-sources.json
```

**Extract all dependency licenses:**
```bash
cratelist Cargo.lock --license-contents ~/.cargo/registry/src/index.crates.io-HASH > DEPENDENCIES_LICENSE
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
