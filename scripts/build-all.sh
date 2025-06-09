#!/bin/bash
# Unified build script for all platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    echo "Usage: $0 [PLATFORM] [BUILD_MODE]"
    echo ""
    echo "PLATFORM:"
    echo "  macos     - Build for macOS (native)"
    echo "  linux     - Build for Linux"
    echo "  windows   - Build for Windows"
    echo "  current   - Build for current platform (default)"
    echo ""
    echo "BUILD_MODE:"
    echo "  debug     - Debug build"
    echo "  release   - Release build (default)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Build current platform in release mode"
    echo "  $0 macos             # Build macOS in release mode"
    echo "  $0 linux debug       # Build Linux in debug mode"
}

PLATFORM="${1:-current}"
BUILD_MODE="${2:-release}"

if [ "$PLATFORM" = "--help" ] || [ "$PLATFORM" = "-h" ]; then
    show_help
    exit 0
fi

print_status "Building SeeU Desktop"
print_status "Platform: $PLATFORM"
print_status "Build mode: $BUILD_MODE"

# Create output directory
mkdir -p dist

# Function to build for specific platform
build_platform() {
    local platform=$1
    local mode=$2
    
    case "$platform" in
        "macos")
            print_status "Building for macOS..."
            if [ -f "scripts/build-macos-native.sh" ]; then
                ./scripts/build-macos-native.sh "$mode"
            else
                print_error "macOS build script not found"
                return 1
            fi
            ;;
        "linux")
            print_status "Building for Linux..."
            if [ -f "scripts/build-linux.sh" ]; then
                ./scripts/build-linux.sh "$mode"
            else
                print_error "Linux build script not found"
                return 1
            fi
            ;;
        "windows")
            print_status "Building for Windows..."
            if [ -f "scripts/build-windows.sh" ]; then
                ./scripts/build-windows.sh "$mode"
            else
                print_error "Windows build script not found"
                return 1
            fi
            ;;
        "current")
            # Detect current OS and build native
            OS="$(uname)"
            case "$OS" in
                "Darwin")
                    build_platform "macos" "$mode"
                    ;;
                "Linux")
                    build_platform "linux" "$mode"
                    ;;
                "MINGW"*|"MSYS"*|"CYGWIN"*)
                    build_platform "windows" "$mode"
                    ;;
                *)
                    print_warning "Unknown OS: $OS, trying generic build..."
                    local flags=""
                    if [ "$mode" = "release" ]; then
                        flags="--release"
                    fi
                    
                    if cargo build $flags; then
                        print_success "Generic build completed"
                        # Copy binary to dist
                        local binary_path="target"
                        if [ "$mode" = "release" ]; then
                            binary_path="$binary_path/release"
                        else
                            binary_path="$binary_path/debug"
                        fi
                        
                        if [ -f "$binary_path/seeu_desktop" ]; then
                            cp "$binary_path/seeu_desktop" "dist/"
                            print_success "Binary copied to dist/"
                        elif [ -f "$binary_path/seeu_desktop.exe" ]; then
                            cp "$binary_path/seeu_desktop.exe" "dist/"
                            print_success "Binary copied to dist/"
                        fi
                    else
                        print_error "Generic build failed"
                        return 1
                    fi
                    ;;
            esac
            ;;
        *)
            print_error "Unknown platform: $platform"
            return 1
            ;;
    esac
}

# Build based on platform selection
build_platform "$PLATFORM" "$BUILD_MODE"

print_success "Build completed successfully!"
print_status "Binaries available in dist/ directory"
