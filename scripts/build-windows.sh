#!/bin/bash
# Build script for Windows targets using vcpkg with fallback support

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

# Parse command line arguments
STATIC_LINK="true"
BUILD_MODE="release"
TARGET_ARCH=""
INCLUDE_32BIT="false"

# Support both positional and named arguments
if [[ $# -gt 0 ]] && [[ "$1" != --* ]]; then
    # First argument is build mode if it doesn't start with --
    BUILD_MODE="$1"
    shift
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        --static)
            STATIC_LINK="$2"
            shift 2
            ;;
        --mode)
            BUILD_MODE="$2"
            shift 2
            ;;
        --arch)
            TARGET_ARCH="$2"
            shift 2
            ;;
        --include-32bit)
            INCLUDE_32BIT="true"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [BUILD_MODE] [OPTIONS]"
            echo ""
            echo "BUILD_MODE: debug or release (default: release)"
            echo ""
            echo "Options:"
            echo "  --static BOOL       Enable static linking (true/false) [default: true]"
            echo "  --mode MODE         Build mode (debug/release) [default: release]"
            echo "  --arch ARCH         Target architecture (x64/x86/all) [default: x64]"
            echo "  --include-32bit     Include 32-bit build when arch=all"
            echo "  -h, --help          Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                  # Build in release mode"
            echo "  $0 debug            # Build in debug mode"
            echo "  $0 --mode debug     # Build in debug mode (alternative syntax)"
            exit 0
            ;;
        debug|release)
            BUILD_MODE="$1"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set default architecture
if [ -z "$TARGET_ARCH" ]; then
    TARGET_ARCH="x64"
fi

# Create output directory
mkdir -p dist/windows

print_status "Building SeeU Desktop for Windows..."
print_status "Static linking: $STATIC_LINK"
print_status "Build mode: $BUILD_MODE"
print_status "Target architecture: $TARGET_ARCH"

# Function to build with vcpkg
build_with_vcpkg() {
    local target="$1"
    local rust_target="$2"
    local vcpkg_triplet="$3"
    local output_dir="$4"
    
    print_status "Building $target using vcpkg..."
    
    # Install Rust target
    rustup target add "$rust_target"
    
    # Set vcpkg environment
    export VCPKG_DEFAULT_TRIPLET="$vcpkg_triplet"
    
    # Build flags
    local build_flags="--target $rust_target"
    if [ "$BUILD_MODE" = "release" ]; then
        build_flags="$build_flags --release"
    fi
    
    # Build the project
    print_status "Running cargo build $build_flags..."
    cargo build $build_flags
    
    # Determine binary path
    local binary_path="target/$rust_target"
    if [ "$BUILD_MODE" = "release" ]; then
        binary_path="$binary_path/release"
    else
        binary_path="$binary_path/debug"
    fi
    
    # Copy binary
    if [ -f "$binary_path/seeu_desktop.exe" ]; then
        mkdir -p "$output_dir"
        cp "$binary_path/seeu_desktop.exe" "$output_dir/"
        print_success "Binary copied to $output_dir/seeu_desktop.exe"
        
        # Copy assets
        if [ -d "assets" ]; then
            cp -r assets "$output_dir/"
            print_status "Assets copied to $output_dir/"
        fi
        
        # Create batch launcher script
        cat > "$output_dir/seeu-desktop.bat" << 'EOF'
@echo off
cd /d "%~dp0"
seeu_desktop.exe %*
EOF
        print_status "Batch launcher script created"
        
        return 0
    else
        print_error "Binary not found at $binary_path/seeu_desktop.exe"
        return 1
    fi
}

# Check if vcpkg is available and try vcpkg build first
if [ -n "$VCPKG_ROOT" ] && ([ -f "$VCPKG_ROOT/vcpkg" ] || [ -f "$VCPKG_ROOT/vcpkg.exe" ]); then
    print_success "vcpkg found at $VCPKG_ROOT"
    
    # Build based on target architecture
    case "$TARGET_ARCH" in
        "x64")
            if [ "$STATIC_LINK" = "true" ]; then
                build_with_vcpkg "Windows x64 (static)" "x86_64-pc-windows-msvc" "x64-windows-static" "dist/windows"
            else
                build_with_vcpkg "Windows x64" "x86_64-pc-windows-msvc" "x64-windows" "dist/windows"
            fi
            ;;
        "x86")
            if [ "$STATIC_LINK" = "true" ]; then
                build_with_vcpkg "Windows x86 (static)" "i686-pc-windows-msvc" "x86-windows-static" "dist/windows"
            else
                build_with_vcpkg "Windows x86" "i686-pc-windows-msvc" "x86-windows" "dist/windows"
            fi
            ;;
        "all")
            # Build x64 (primary)
            if [ "$STATIC_LINK" = "true" ]; then
                build_with_vcpkg "Windows x64 (static)" "x86_64-pc-windows-msvc" "x64-windows-static" "dist/windows/x64"
            else
                build_with_vcpkg "Windows x64" "x86_64-pc-windows-msvc" "x64-windows" "dist/windows/x64"
            fi
            
            # Also copy to main directory for compatibility
            cp dist/windows/x64/seeu_desktop.exe dist/windows/
            cp dist/windows/x64/seeu-desktop.bat dist/windows/
            if [ -d "dist/windows/x64/assets" ]; then
                cp -r dist/windows/x64/assets dist/windows/
            fi
            
            # Build x86 if requested
            if [ "$INCLUDE_32BIT" = "true" ]; then
                if [ "$STATIC_LINK" = "true" ]; then
                    build_with_vcpkg "Windows x86 (static)" "i686-pc-windows-msvc" "x86-windows-static" "dist/windows/x86"
                else
                    build_with_vcpkg "Windows x86" "i686-pc-windows-msvc" "x86-windows" "dist/windows/x86"
                fi
            fi
            ;;
        *)
            print_error "Invalid architecture: $TARGET_ARCH"
            exit 1
            ;;
    esac
    
    print_success "vcpkg build completed successfully!"
    
elif [[ "$(uname)" == MINGW* ]] || [[ "$(uname)" == MSYS* ]] || [[ "$(uname)" == CYGWIN* ]]; then
    # Native build on Windows (fallback)
    print_warning "vcpkg not available, using native Windows build..."
    
    # Determine Rust target based on architecture
    case "$TARGET_ARCH" in
        "x64")
            RUST_TARGET="x86_64-pc-windows-msvc"
            ;;
        "x86")
            RUST_TARGET="i686-pc-windows-msvc"
            ;;
        "all")
            RUST_TARGET="x86_64-pc-windows-msvc"  # Default to x64
            ;;
        *)
            print_error "Invalid architecture: $TARGET_ARCH"
            exit 1
            ;;
    esac
    
    # Install Rust target
    rustup target add "$RUST_TARGET"
    
    # Build flags
    build_flags="--target $RUST_TARGET"
    if [ "$BUILD_MODE" = "release" ]; then
        build_flags="$build_flags --release"
    fi
    
    print_status "Building for Windows (native) with target: $RUST_TARGET"
    cargo build $build_flags
    
    # Copy binary
    binary_path="target/$RUST_TARGET"
    if [ "$BUILD_MODE" = "release" ]; then
        binary_path="$binary_path/release"
    else
        binary_path="$binary_path/debug"
    fi
    
    if [ -f "$binary_path/seeu_desktop.exe" ]; then
        cp "$binary_path/seeu_desktop.exe" dist/windows/
        print_success "Binary copied to dist/windows/seeu_desktop.exe"
    else
        print_error "Binary not found at $binary_path/seeu_desktop.exe"
        exit 1
    fi
    
    # Copy assets and create launcher
    if [ -d "assets" ]; then
        cp -r assets dist/windows/
    fi
    
    cat > dist/windows/seeu-desktop.bat << 'EOF'
@echo off
cd /d "%~dp0"
seeu_desktop.exe %*
EOF
    
    print_success "Native Windows build completed!"
    
elif command -v docker &> /dev/null; then
    # Docker-based build (fallback)
    print_warning "vcpkg not available and not on Windows, using Docker build..."
    
    # Determine Rust target for Docker build
    case "$TARGET_ARCH" in
        "x64")
            RUST_TARGET="x86_64-pc-windows-gnu"
            ;;
        "x86")
            RUST_TARGET="i686-pc-windows-gnu"
            ;;
        "all")
            RUST_TARGET="x86_64-pc-windows-gnu"  # Default to x64
            ;;
        *)
            print_error "Invalid architecture: $TARGET_ARCH"
            exit 1
            ;;
    esac
    
    # Create a Dockerfile for Windows build
    cat > Dockerfile.windows << EOF
FROM rust:slim

# Install required dependencies for Windows cross-compilation
RUN apt-get update && apt-get install -y --no-install-recommends \\
    mingw-w64 \\
    && rm -rf /var/lib/apt/lists/*

# Add Windows target
RUN rustup target add $RUST_TARGET

WORKDIR /app
COPY . .

# Build the application for Windows
RUN cargo build --release --target $RUST_TARGET

# Create the output directory
RUN mkdir -p /output
RUN cp target/$RUST_TARGET/release/seeu_desktop.exe /output/
EOF

    # Build with Docker
    print_status "Building Docker image for Windows..."
    docker build -t seeu-desktop-windows-builder -f Dockerfile.windows .
    
    # Extract binary
    print_status "Extracting binary from Docker container..."
    docker create --name seeu-temp-container-win seeu-desktop-windows-builder
    docker cp seeu-temp-container-win:/output/seeu_desktop.exe dist/windows/
    docker rm seeu-temp-container-win
    
    # Clean up
    docker rmi seeu-desktop-windows-builder
    rm Dockerfile.windows
    
    # Copy assets and create launcher
    if [ -d "assets" ]; then
        cp -r assets dist/windows/
    fi
    
    cat > dist/windows/seeu-desktop.bat << 'EOF'
@echo off
cd /d "%~dp0"
seeu_desktop.exe %*
EOF
    
    print_success "Docker build completed!"
    
else
    print_error "Cannot build for Windows. Please:"
    print_error "1. Install vcpkg using ./scripts/setup-vcpkg.sh, or"
    print_error "2. Build on a Windows system, or"
    print_error "3. Install Docker for cross-platform builds"
    exit 1
fi

print_success "Windows build completed: dist/windows/seeu_desktop.exe"
print_status "You can run the application with: ./dist/windows/seeu-desktop.bat"
