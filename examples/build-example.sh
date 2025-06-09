#!/bin/bash
# Example script demonstrating vcpkg-based cross-platform builds

set -e

echo "=== SeeU Desktop vcpkg Build Example ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Step 1: Setup vcpkg (if not already done)
print_step "1. Setting up vcpkg..."
if [ -z "$VCPKG_ROOT" ]; then
    print_info "vcpkg not detected, running setup..."
    ./scripts/setup-vcpkg.sh
    
    # Source the environment
    if [ -f "$HOME/.bashrc" ]; then
        source "$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        source "$HOME/.zshrc"
    fi
else
    print_success "vcpkg already configured at $VCPKG_ROOT"
fi

# Step 2: Test vcpkg configuration
print_step "2. Testing vcpkg configuration..."
./scripts/test-vcpkg.sh

# Step 3: Build for current platform
print_step "3. Building for current platform..."
CURRENT_OS="$(uname)"
case "$CURRENT_OS" in
    "Linux")
        ARCH="$(uname -m)"
        if [ "$ARCH" = "x86_64" ]; then
            TARGET="linux-x64"
        elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
            TARGET="linux-arm64"
        else
            print_info "Unsupported architecture: $ARCH, defaulting to linux-x64"
            TARGET="linux-x64"
        fi
        ;;
    "Darwin")
        print_info "macOS detected, but vcpkg build not yet configured for macOS"
        print_info "Using traditional build method..."
        ./scripts/build-macos.sh
        exit 0
        ;;
    "MINGW"*|"MSYS"*|"CYGWIN"*)
        TARGET="windows-x64"
        ;;
    *)
        print_info "Unknown OS: $CURRENT_OS, defaulting to linux-x64"
        TARGET="linux-x64"
        ;;
esac

print_info "Building for target: $TARGET"
./scripts/build-vcpkg.sh --target "$TARGET" --mode release --static true

# Step 4: Show results
print_step "4. Build results:"
if [ -d "dist" ]; then
    echo "Built binaries:"
    find dist -name "seeu_desktop*" -type f | while read -r file; do
        size=$(du -h "$file" | cut -f1)
        echo "  - $file ($size)"
    done
else
    print_info "No dist directory found"
fi

print_success "Build example completed successfully!"
echo ""
echo "Next steps:"
echo "  - Run the binary: ./dist/$TARGET/seeu_desktop"
echo "  - Build for other platforms: ./scripts/build-vcpkg.sh --target all"
echo "  - Read the full guide: docs/VCPKG_BUILD_GUIDE.md"
