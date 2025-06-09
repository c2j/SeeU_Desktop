#!/bin/bash
# Build script for Linux targets using vcpkg with fallback support

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
INCLUDE_ARM64="false"

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
        --include-arm64)
            INCLUDE_ARM64="true"
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
            echo "  --arch ARCH         Target architecture (x64/arm64/all) [default: auto]"
            echo "  --include-arm64     Include ARM64 build when arch=all"
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

# Create output directory
mkdir -p dist/linux

print_status "Building SeeU Desktop for Linux..."
print_status "Static linking: $STATIC_LINK"
print_status "Build mode: $BUILD_MODE"

# Function to build with vcpkg
build_with_vcpkg() {
    local target="$1"
    local rust_target="$2"
    local vcpkg_triplet="$3"
    local output_dir="$4"

    print_status "Building $target using vcpkg..."

    # Install Rust target
    rustup target add "$rust_target"

    # Setup build environment to avoid pkg-config
    source scripts/setup-build-env.sh "$rust_target"

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
    if [ -f "$binary_path/seeu_desktop" ]; then
        mkdir -p "$output_dir"
        cp "$binary_path/seeu_desktop" "$output_dir/"
        print_success "Binary copied to $output_dir/seeu_desktop"
        
        # Copy assets
        if [ -d "assets" ]; then
            cp -r assets "$output_dir/"
            print_status "Assets copied to $output_dir/"
        fi
        
        # Create launcher script
        cat > "$output_dir/seeu-desktop.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./seeu_desktop "$@"
EOF
        chmod +x "$output_dir/seeu-desktop.sh"
        print_status "Launcher script created"
        
        return 0
    else
        print_error "Binary not found at $binary_path/seeu_desktop"
        return 1
    fi
}

# Check if vcpkg is available and try vcpkg build first
if [ -n "$VCPKG_ROOT" ] && [ -f "$VCPKG_ROOT/vcpkg" ]; then
    print_success "vcpkg found at $VCPKG_ROOT"
    
    # Determine target architecture
    if [ -z "$TARGET_ARCH" ]; then
        CURRENT_ARCH="$(uname -m)"
        case "$CURRENT_ARCH" in
            "x86_64")
                TARGET_ARCH="x64"
                ;;
            "aarch64"|"arm64")
                TARGET_ARCH="arm64"
                ;;
            *)
                print_warning "Unknown architecture: $CURRENT_ARCH, defaulting to x64"
                TARGET_ARCH="x64"
                ;;
        esac
    fi
    
    # Build based on target architecture
    case "$TARGET_ARCH" in
        "x64")
            if [ "$STATIC_LINK" = "true" ]; then
                build_with_vcpkg "Linux x64 (static)" "x86_64-unknown-linux-musl" "x64-linux" "dist/linux"
            else
                build_with_vcpkg "Linux x64" "x86_64-unknown-linux-gnu" "x64-linux" "dist/linux"
            fi
            ;;
        "arm64")
            if [ "$STATIC_LINK" = "true" ]; then
                build_with_vcpkg "Linux ARM64 (static)" "aarch64-unknown-linux-musl" "arm64-linux" "dist/linux"
            else
                build_with_vcpkg "Linux ARM64" "aarch64-unknown-linux-gnu" "arm64-linux" "dist/linux"
            fi
            ;;
        "all")
            # Build x64
            if [ "$STATIC_LINK" = "true" ]; then
                build_with_vcpkg "Linux x64 (static)" "x86_64-unknown-linux-musl" "x64-linux" "dist/linux/x64"
            else
                build_with_vcpkg "Linux x64" "x86_64-unknown-linux-gnu" "x64-linux" "dist/linux/x64"
            fi
            
            # Also copy to main directory for compatibility
            cp dist/linux/x64/seeu_desktop dist/linux/
            cp dist/linux/x64/seeu-desktop.sh dist/linux/
            if [ -d "dist/linux/x64/assets" ]; then
                cp -r dist/linux/x64/assets dist/linux/
            fi
            
            # Build ARM64 if requested
            if [ "$INCLUDE_ARM64" = "true" ]; then
                if [ "$STATIC_LINK" = "true" ]; then
                    build_with_vcpkg "Linux ARM64 (static)" "aarch64-unknown-linux-musl" "arm64-linux" "dist/linux/arm64"
                else
                    build_with_vcpkg "Linux ARM64" "aarch64-unknown-linux-gnu" "arm64-linux" "dist/linux/arm64"
                fi
            fi
            ;;
        *)
            print_error "Invalid architecture: $TARGET_ARCH"
            exit 1
            ;;
    esac
    
    print_success "vcpkg build completed successfully!"
    
elif [ "$(uname)" = "Linux" ]; then
    # Native build on Linux (fallback)
    print_warning "vcpkg not available, using native Linux build..."
    
    # Install Rust target for static linking if requested
    if [ "$STATIC_LINK" = "true" ]; then
        rustup target add x86_64-unknown-linux-musl
        TARGET="x86_64-unknown-linux-musl"
    else
        TARGET="x86_64-unknown-linux-gnu"
    fi
    
    # Build flags
    build_flags=""
    if [ "$BUILD_MODE" = "release" ]; then
        build_flags="--release"
    fi
    if [ -n "$TARGET" ]; then
        build_flags="$build_flags --target $TARGET"
    fi
    
    print_status "Building for Linux (native) with flags: $build_flags"
    cargo build $build_flags
    
    # Copy binary
    if [ -n "$TARGET" ]; then
        binary_path="target/$TARGET"
    else
        binary_path="target"
    fi
    
    if [ "$BUILD_MODE" = "release" ]; then
        binary_path="$binary_path/release"
    else
        binary_path="$binary_path/debug"
    fi
    
    if [ -f "$binary_path/seeu_desktop" ]; then
        cp "$binary_path/seeu_desktop" dist/linux/
        print_success "Binary copied to dist/linux/seeu_desktop"
    else
        print_error "Binary not found at $binary_path/seeu_desktop"
        exit 1
    fi
    
    # Copy assets and create launcher
    if [ -d "assets" ]; then
        cp -r assets dist/linux/
    fi
    
    cat > dist/linux/seeu-desktop.sh << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./seeu_desktop "$@"
EOF
    chmod +x dist/linux/seeu-desktop.sh
    
    print_success "Native Linux build completed!"
    
elif command -v docker &> /dev/null; then
    # Docker-based build (fallback)
    print_warning "vcpkg not available and not on Linux, using Docker build..."
    
    # Create a Dockerfile for Linux build
    cat > Dockerfile.linux << 'EOF'
# #FROM alpine:edge
# #FROM kuyoh/vcpkg:latest-alpine24.04
# FROM csantve/alpine-vcpkg

# # Install dependencies
# RUN apk update && apk add --no-cache \
#     musl-dev \
#     openssl-dev \
#     pkgconf \
#     cargo \
#     rustup \
#     curl \
#     ca-certificates \
#     build-base \
#     linux-headers \
#     libc-dev
# # Initialize rustup
# RUN rustup-init -y --default-toolchain stable
# ENV PATH="/root/.cargo/bin:${PATH}"
# # Add Rust targets
# RUN rustup target add x86_64-unknown-linux-musl x86_64-unknown-linux-gnu
# RUN apk update && apk add --no-cache build-base gcc
# RUN apk update && apk add --no-cache glib-dev
# RUN apk update && apk add --no-cache gtk+3.0-dev
# RUN apk update && apk add --no-cache perl

FROM ubuntu:20.04
RUN  apt-get update && apt-get install -y --no-install-recommends tzdata   && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    curl ca-certificates  \
    libgtk-3-dev libappindicator3-dev librsvg2-dev libwebkit2gtk-4.0-dev \
    && rm -rf /var/lib/apt/lists/*

# Install latest Rust with PATH properly set for the current shell
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $HOME/.cargo/env \
    && cargo -V
ENV PATH="/root/.cargo/bin:${PATH}"

#RUN  sed -i 's|http://archive.ubuntu.com/ubuntu|http://mirrors.aliyun.com/ubuntu|g' /etc/apt/sources.list &&  apt update
RUN  apt-get update && apt-get install -y --no-install-recommends musl-dev musl-tools   && rm -rf /var/lib/apt/lists/*
RUN  apt-get update && apt-get install -y --no-install-recommends clang libxkbcommon-x11-dev pkg-config libvulkan-dev libwayland-dev xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev   && rm -rf /var/lib/apt/lists/*

# 添加目标平台 x86_64-unknown-linux-musl
RUN rustup target add x86_64-unknown-linux-musl  x86_64-unknown-linux-gnu

# 将glibc升级到2.28
RUN  apt-get update && apt-get install -y --no-install-recommends gawk \
    bison \
    make \
    gcc \
    g++ \
    python3 \
    texinfo   && rm -rf /var/lib/apt/lists/*

# RUN curl -o glibc-2.28.tar.gz http://ftp.gnu.org/gnu/libc/glibc-2.28.tar.gz && \
#     tar -xzf glibc-2.28.tar.gz && \
#     mkdir glibc-2.28-build && cd glibc-2.28-build && \
#     ../glibc-2.28/configure --prefix=/opt/glibc-2.28 && \
#     make && \
#     make install
RUN curl -o glibc-2.35.tar.gz http://ftp.gnu.org/gnu/libc/glibc-2.35.tar.gz && \
    tar -xzf glibc-2.35.tar.gz && \
    mkdir glibc-2.35-build && cd glibc-2.35-build && \
    ../glibc-2.35/configure --prefix=/opt/glibc-2.35 && \
    make && \
    make install
#ENV LIBRARY_PATH='/opt/glibc-2.35/lib:$LIBRARY_PATH'

WORKDIR /app
COPY . .


# Build the application
# LD_LIBRARY_PATH=/opt/glibc-2.28/lib:$LD_LIBRARY_PATH cargo build --release
# LIBRARY_PATH=/opt/glibc-2.28/lib:$LIBRARY_PATH cargo build --release
# LIBRARY_PATH=/opt/glibc-2.35/lib:$LIBRARY_PATH cargo build --release

# RUN if [ "$STATIC_LINK" = "true" ]; then \
#         PKG_CONFIG_PATH=$VCPKG_ROOT/installed/x86_64-unknown-linux-musl/lib/pkgconfig \
#         PKG_CONFIG_ALLOW_CROSS=1 \
#         CC="gcc -fPIE -pie" \
#         cargo build --release --target x86_64-unknown-linux-gnu; \
#         cp target/x86_64-unknown-linux-musl/release/seeu_desktop /output/; \
#     else \
#          CC="gcc -fPIE -pie" \
#          cargo build --release --target x86_64-unknown-linux-gnu; \
#         cp target/x86_64-unknown-linux-gnu/release/seeu_desktop /output/; \
#     fi

# Create output directory
# RUN mkdir -p /output
EOF

    # Build with Docker
    print_status "Building Docker image..."
    docker build --build-arg STATIC_LINK="$STATIC_LINK" -t seeu-desktop-linux-builder -f Dockerfile.linux .
    
    # Extract binary
    print_status "Extracting binary from Docker container..."
    docker create --name seeu-temp-container seeu-desktop-linux-builder
    docker cp seeu-temp-container:/output/seeu_desktop dist/linux/
    docker rm seeu-temp-container
    
    # Clean up
    # docker rmi seeu-desktop-linux-builder
    rm Dockerfile.linux
    
    # Copy assets and create launcher
    if [ -d "assets" ]; then
        cp -r assets dist/linux/
    fi
    
    cat > dist/linux/seeu-desktop.sh << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./seeu_desktop "$@"
EOF
    chmod +x dist/linux/seeu-desktop.sh
    
    print_success "Docker build completed!"
    
else
    print_error "Cannot build for Linux. Please:"
    print_error "1. Install vcpkg using ./scripts/setup-vcpkg.sh, or"
    print_error "2. Build on a Linux system, or"
    print_error "3. Install Docker for cross-platform builds"
    exit 1
fi

print_success "Linux build completed: dist/linux/seeu_desktop"
print_status "You can run the application with: ./dist/linux/seeu-desktop.sh"
