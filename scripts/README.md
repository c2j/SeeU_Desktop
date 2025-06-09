# SeeU Desktop Build Scripts

This directory contains the build scripts for SeeU Desktop application.

## Quick Start

### Build for Current Platform
```bash
# Build in release mode for current platform
./scripts/build-all.sh

# Build in debug mode
./scripts/build-all.sh current debug
```

### Build for Specific Platform
```bash
# Build for macOS
./scripts/build-all.sh macos

# Build for Linux
./scripts/build-all.sh linux

# Build for Windows
./scripts/build-all.sh windows
```

## Available Scripts

### Core Build Scripts

- **`build-all.sh`** - Unified build script for all platforms
  - Usage: `./build-all.sh [PLATFORM] [BUILD_MODE]`
  - Platforms: `macos`, `linux`, `windows`, `current` (default)
  - Build modes: `debug`, `release` (default)

- **`build-macos-native.sh`** - Native macOS build (avoids GTK dependencies)
  - Usage: `./build-macos-native.sh [BUILD_MODE]`
  - Creates app bundle for release builds

- **`build-linux.sh`** - Linux build with vcpkg support
  - Usage: `./build-linux.sh [BUILD_MODE]`
  - Supports both musl and glibc targets

- **`build-windows.sh`** - Windows build script
  - Usage: `./build-windows.sh [BUILD_MODE]`
  - Supports both 32-bit and 64-bit builds

### Setup Scripts

- **`setup-vcpkg.sh`** - Install and configure vcpkg
  - Installs vcpkg to `$HOME/vcpkg`
  - Configures environment variables
  - Installs basic packages (openssl, sqlite3, zlib)

- **`setup-build-env.sh`** - Setup build environment
  - Configures environment variables for cross-compilation
  - Sets up vcpkg integration
  - Installs Rust targets

## Build Requirements

### General
- Rust 1.70+ with cargo
- Git

### Platform-Specific

#### macOS
- Xcode Command Line Tools
- Homebrew (for dependencies)

#### Linux
- GCC or Clang
- pkg-config
- Development libraries (if not using vcpkg)

#### Windows
- Visual Studio Build Tools
- Windows SDK

## vcpkg Integration

The project uses vcpkg for dependency management to avoid pkg-config issues:

1. **Setup vcpkg** (one-time):
   ```bash
   ./scripts/setup-vcpkg.sh
   ```

2. **Build with vcpkg**:
   ```bash
   export VCPKG_ROOT="$HOME/vcpkg"
   ./scripts/build-all.sh linux
   ```

## Output

All builds output to the `dist/` directory:
- `dist/macos/` - macOS binaries and app bundle
- `dist/linux/` - Linux binaries
- `dist/windows/` - Windows binaries

## Troubleshooting

### Common Issues

1. **GTK/pkg-config errors**: Use the native build scripts which avoid GTK dependencies
2. **Missing vcpkg**: Run `./scripts/setup-vcpkg.sh` first
3. **Cross-compilation issues**: Build on the target platform or use Docker

### Environment Variables

Key environment variables used by the build system:
- `VCPKG_ROOT` - Path to vcpkg installation
- `PKG_CONFIG_ALLOW_CROSS` - Set to 0 to disable pkg-config
- `OPENSSL_STATIC` - Set to 1 for static OpenSSL linking

## Examples

```bash
# Quick development build
./scripts/build-all.sh current debug

# Production macOS build with app bundle
./scripts/build-all.sh macos release

# Linux build with vcpkg (after setup)
export VCPKG_ROOT="$HOME/vcpkg"
./scripts/build-all.sh linux release

# Setup everything and build
./scripts/setup-vcpkg.sh
./scripts/setup-build-env.sh
./scripts/build-all.sh
```
