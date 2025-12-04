# SPC Version

A simple CLI tool for reading and downloading pre-built PHP binaries using the [Static PHP CLI](https://github.com/crazywhalecc/static-php-cli) tool

## Quick Start

```bash
# Get the latest PHP version available
spc-utils latest
spc-utils latest -C minimal -V 8.2
spc-utils latest -C bulk --os linux -V 8.2 --no-cache

# Check if you need to update
spc-utils check-update -V 8.4.10

# Download a binary
spc-utils download -o php
```

## Installation

### From crates.io

```bash
cargo install spc-utils
```

## Commands

### latest

Fetch the latest available PHP version for your platform.

```bash
# Basic usage - auto-detects your OS and architecture
spc-utils latest

# Get latest version for a specific build category
spc-utils latest -C common

# Get latest 8.4.x version
spc-utils latest -V 8.4

# Combine options
spc-utils latest -C bulk -V 8.4 -O linux -A x86_64
```

| Option | Description |
|--------|-------------|
| `-C, --category` | Build category: `bulk`, `common`, `minimal`, `win-min`, `win-max` |
| `-V, --version` | Filter by major.minor version (e.g., `8.4` or `8.3`) |
| `-O` | Target OS: `linux`, `macos`, `windows` |
| `-A, --arch` | Architecture: `x86_64`, `aarch64` |
| `-B, --build-type` | Build type: `cli`, `fpm`, `micro` |
| `--no-cache` | Bypass cache and fetch fresh data from the API |

### check-update

Check if your installed PHP version is current. Shows the download URL when an update is available.

```bash
# Check if 8.4.10 is the latest
spc-utils check-update -V 8.4.10

# Check against a specific category
spc-utils check-update -C common -V 8.4.10

# Version shorthand works too
spc-utils check-update -V 8.4
```

Output examples:

```
You have the latest version: 8.4.15 (cached)
```

```
Update available: 8.4.10 -> 8.4.15 (cached)
  https://dl.static-php.dev/static-php-cli/bulk/macos-aarch64/php-8.4.15-cli
```

| Option | Description |
|--------|-------------|
| `-V, --version` | Your current version (required) |
| `-C, --category` | Build category to check against |
| `--no-cache` | Bypass cache and fetch fresh data |

### download

Download a Static PHP CLI binary to your local machine.

```bash
# Download latest to ./php
spc-utils download -o php

# Download specific version
spc-utils download -V 8.4.10 -o php

# Download for a different platform
spc-utils download -O linux -A x86_64 -o ./bin/php

# Download the common build
spc-utils download -C common -V 8.4 -o php-common
```

| Option | Description |
|--------|-------------|
| `-o, --output` | Output file path (required) |
| `-C, --category` | Build category |
| `-V, --version` | PHP version to download |
| `-O` | Target OS |
| `-A, --arch` | Architecture |
| `-B, --build-type` | Build type: `cli`, `fpm`, `micro` |
| `--no-cache` | Bypass cache when resolving version |

### cache

Manage locally cached API responses. Caching avoids repeated API calls and speeds up subsequent commands.

```bash
# View cached files with details
spc-utils cache list

# Clear all cached data
spc-utils cache clear

# Clear cache for a specific category only
spc-utils cache clear -C bulk

# Get the cache directory path
spc-utils cache path
```

Example `cache list` output:

```
+----------+---------+---------+------------------+-----------+
| Category | Entries | Size    | Modified         | Expires   |
+----------+---------+---------+------------------+-----------+
| bulk     | 661     | 115.2KB | 2025-01-15 10:30 | in 23h 45m|
| common   | 312     | 52.1KB  | 2025-01-15 09:15 | in 22h 30m|
+----------+---------+---------+------------------+-----------+
```

### usage examples

Display usage examples for all commands.

```bash
spc-utils examples
```

## Build Categories

Static PHP CLI offers different build configurations with varying extension sets:

| Category | Platform | Description |
|----------|----------|-------------|
| `bulk` | Linux, macOS | Full-featured build with many extensions (default) |
| `common` | Linux, macOS | Common extensions for typical web applications |
| `minimal` | Linux, macOS | Minimal set of core extensions |
| `win-min` | Windows | Windows minimal build |
| `win-max` | Windows | Windows full build (default on Windows) |

## CI/CD Usage

This tool is designed for automating PHP environment setup in CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Install PHP
  run: |
    cargo install spc-utils
    spc-utils download -V 8.4 -o php
    chmod +x php
    ./php --version
```
_note: You must have cargo's bin directory in your PATH for this to work_

```bash
# Shell script example
#!/bin/bash
CURRENT_VERSION=$(./php -v | head -1 | cut -d' ' -f2)
spc-utils check-update -V "$CURRENT_VERSION" || spc-utils download -o php
```

## Caching

API responses are cached locally (TTL = end of day) to minimize network requests. Cache files are stored in your system's standard cache directory:

- Linux: `~/.cache/spc-utils/`
- macOS: `~/Library/Caches/spc-utils/`
- Windows: `%LOCALAPPDATA%\spc-utils\`

Use `--no-cache` on any command to bypass the cache and fetch fresh data.

## License

MIT
