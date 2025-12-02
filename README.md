# SPC Utils

CLI tool for fetching and downloading [Static PHP CLI](https://static-php.dev) binaries.

## Installation

```bash
cargo install --path .
```

## Commands

### `latest`

Fetch the latest available version for your platform.

```bash
spc-utils latest
spc-utils latest -C bulk
spc-utils latest -C common -V 8.4
spc-utils latest --no-cache
```

**Options:**
- `-C, --category` — Build category: `bulk`, `common`, `minimal`, `win-min`, `win-max`
- `-V, --version` — Filter by major.minor version (e.g., `8.4`)
- `-O` — Target OS: `linux`, `macos`, `windows`
- `-A, --arch` — Architecture: `x86_64`, `aarch64`
- `-B, --build-type` — Build type: `cli`, `fpm`, `micro`
- `--no-cache` — Skip cache and fetch fresh data

### `check-update`

Check if your current version is the latest.

```bash
spc-utils check-update -v 8.4.10
spc-utils check-update -C common -v 8.4.10
```

**Options:**
- `-v, --version` — Version to check (required)
- `-C, --category` — Build category
- `--no-cache` — Skip cache and fetch fresh data

### `download`

Download a Static PHP CLI binary.

```bash
spc-utils download -o php
spc-utils download -C bulk -V 8.4.10 -o php
spc-utils download -C common -V 8.4 -O linux -A x86_64 -o ./php-binary
```

**Options:**
- `-o, --output` — Output file path (required)
- `-C, --category` — Build category
- `-V, --version` — PHP version
- `-O` — Target OS
- `-A, --arch` — Architecture
- `-B, --build-type` — Build type
- `--no-cache` — Skip cache and fetch fresh data

### `cache`

Manage locally cached API responses.

```bash
spc-utils cache list          # Show cached files
spc-utils cache clear         # Clear all cache
spc-utils cache clear -C bulk # Clear specific category
spc-utils cache path          # Print cache directory
```

**Subcommands:**
- `list` — Display cached files with size, entry count, and validity
- `clear` — Remove cached files (optionally by category with `-C`)
- `path` — Print the cache directory location

## Build Categories

| Category | Description |
|----------|-------------|
| `bulk` | Full-featured build with many extensions (default on Linux/macOS) |
| `common` | Common extensions for typical web applications |
| `minimal` | Minimal set of core extensions |
| `win-min` | Windows minimal build |
| `win-max` | Windows full build (default on Windows) |

## License

MIT
