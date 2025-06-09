#!/bin/bash
# Setup build environment to avoid pkg-config and use vcpkg/vendored dependencies

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

print_status "Setting up build environment for vcpkg/vendored dependencies..."

# 禁用 pkg-config 相关的环境变量
export PKG_CONFIG_ALLOW_CROSS=0
export PKG_CONFIG=false

# 强制使用 vendored OpenSSL
export OPENSSL_STATIC=1
export OPENSSL_VENDORED=1

# 禁用 GTK 相关的 pkg-config 查找
export GLIB_2_0_NO_PKG_CONFIG=1
export GOBJECT_2_0_NO_PKG_CONFIG=1
export GIO_2_0_NO_PKG_CONFIG=1
export CAIRO_NO_PKG_CONFIG=1
export PANGO_NO_PKG_CONFIG=1
export PANGOCAIRO_NO_PKG_CONFIG=1
export GDK_PIXBUF_2_0_NO_PKG_CONFIG=1
export GTK_3_0_NO_PKG_CONFIG=1
export ATK_NO_PKG_CONFIG=1

# 禁用其他可能的系统库 pkg-config 查找
export FONTCONFIG_NO_PKG_CONFIG=1
export FREETYPE2_NO_PKG_CONFIG=1
export HARFBUZZ_NO_PKG_CONFIG=1

# 强制跳过 glib-sys 的 pkg-config 检查
export GLIB_SYS_NO_PKG_CONFIG=1
export SYSTEM_DEPS_NO_PKG_CONFIG=1

# 设置 vcpkg 相关环境变量
if [ -n "$VCPKG_ROOT" ]; then
    export VCPKG_FEATURE_FLAGS="manifests,versions"
    print_success "vcpkg environment configured: $VCPKG_ROOT"
else
    print_warning "VCPKG_ROOT not set, will use vendored dependencies"
fi

# 根据目标平台设置特定的环境变量
TARGET="${1:-}"
if [ -n "$TARGET" ]; then
    case "$TARGET" in
        "x86_64-unknown-linux-musl"|"aarch64-unknown-linux-musl")
            export VCPKG_DEFAULT_TRIPLET="x64-linux"
            if [[ "$TARGET" == "aarch64"* ]]; then
                export VCPKG_DEFAULT_TRIPLET="arm64-linux"
            fi
            print_status "Configured for musl target: $TARGET"
            ;;
        "x86_64-unknown-linux-gnu"|"aarch64-unknown-linux-gnu")
            export VCPKG_DEFAULT_TRIPLET="x64-linux"
            if [[ "$TARGET" == "aarch64"* ]]; then
                export VCPKG_DEFAULT_TRIPLET="arm64-linux"
            fi
            print_status "Configured for GNU target: $TARGET"
            ;;
        "x86_64-pc-windows-msvc")
            export VCPKG_DEFAULT_TRIPLET="x64-windows-static"
            print_status "Configured for Windows x64 target: $TARGET"
            ;;
        "i686-pc-windows-msvc")
            export VCPKG_DEFAULT_TRIPLET="x86-windows-static"
            print_status "Configured for Windows x86 target: $TARGET"
            ;;
        *)
            print_warning "Unknown target: $TARGET, using default configuration"
            ;;
    esac
fi

# 显示当前环境配置
print_status "Current build environment:"
echo "  PKG_CONFIG_ALLOW_CROSS: $PKG_CONFIG_ALLOW_CROSS"
echo "  PKG_CONFIG: $PKG_CONFIG"
echo "  OPENSSL_STATIC: $OPENSSL_STATIC"
echo "  OPENSSL_VENDORED: $OPENSSL_VENDORED"
echo "  VCPKG_ROOT: ${VCPKG_ROOT:-'(not set)'}"
echo "  VCPKG_DEFAULT_TRIPLET: ${VCPKG_DEFAULT_TRIPLET:-'(not set)'}"

# 创建临时环境文件
ENV_FILE=".build-env"
cat > "$ENV_FILE" << EOF
# Build environment for SeeU Desktop
export PKG_CONFIG_ALLOW_CROSS=0
export PKG_CONFIG=false
export OPENSSL_STATIC=1
export OPENSSL_VENDORED=1
export GLIB_2_0_NO_PKG_CONFIG=1
export GOBJECT_2_0_NO_PKG_CONFIG=1
export GIO_2_0_NO_PKG_CONFIG=1
export CAIRO_NO_PKG_CONFIG=1
export PANGO_NO_PKG_CONFIG=1
export PANGOCAIRO_NO_PKG_CONFIG=1
export GDK_PIXBUF_2_0_NO_PKG_CONFIG=1
export GTK_3_0_NO_PKG_CONFIG=1
export ATK_NO_PKG_CONFIG=1
export FONTCONFIG_NO_PKG_CONFIG=1
export FREETYPE2_NO_PKG_CONFIG=1
export HARFBUZZ_NO_PKG_CONFIG=1
EOF

if [ -n "$VCPKG_ROOT" ]; then
    echo "export VCPKG_ROOT=\"$VCPKG_ROOT\"" >> "$ENV_FILE"
    echo "export VCPKG_FEATURE_FLAGS=\"manifests,versions\"" >> "$ENV_FILE"
fi

if [ -n "$VCPKG_DEFAULT_TRIPLET" ]; then
    echo "export VCPKG_DEFAULT_TRIPLET=\"$VCPKG_DEFAULT_TRIPLET\"" >> "$ENV_FILE"
fi

print_success "Build environment configured successfully!"
print_status "Environment saved to: $ENV_FILE"
print_status "To use this environment in your shell, run: source $ENV_FILE"

# 如果作为脚本运行，导出环境变量
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    print_status "Exporting environment variables for current shell..."
    # 注意：这些变量只在当前脚本中有效
    # 要在调用脚本中使用，需要 source 这个脚本
fi
