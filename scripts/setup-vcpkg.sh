#!/bin/bash
# Simple vcpkg setup script that installs to user directory

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

# 设置 vcpkg 安装目录到用户主目录
VCPKG_DIR="$HOME/vcpkg"

print_status "Setting up vcpkg in user directory: $VCPKG_DIR"

# 检查是否已经安装
if [ -f "$VCPKG_DIR/vcpkg" ]; then
    print_warning "vcpkg is already installed at $VCPKG_DIR"
    export VCPKG_ROOT="$VCPKG_DIR"
    print_success "Using existing vcpkg installation"
else
    # 克隆 vcpkg
    print_status "Cloning vcpkg..."
    if [ -d "$VCPKG_DIR" ]; then
        rm -rf "$VCPKG_DIR"
    fi
    
    git clone https://github.com/Microsoft/vcpkg.git "$VCPKG_DIR"
    cd "$VCPKG_DIR"
    
    # 构建 vcpkg
    print_status "Building vcpkg..."
    ./bootstrap-vcpkg.sh
    
    print_success "vcpkg installed successfully!"
fi

# 设置环境变量
export VCPKG_ROOT="$VCPKG_DIR"
export PATH="$VCPKG_ROOT:$PATH"

# 添加到 shell 配置文件
SHELL_PROFILE=""
if [ -f "$HOME/.zshrc" ]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [ -f "$HOME/.bashrc" ]; then
    SHELL_PROFILE="$HOME/.bashrc"
elif [ -f "$HOME/.bash_profile" ]; then
    SHELL_PROFILE="$HOME/.bash_profile"
elif [ -f "$HOME/.profile" ]; then
    SHELL_PROFILE="$HOME/.profile"
fi

if [ -n "$SHELL_PROFILE" ]; then
    if ! grep -q "VCPKG_ROOT" "$SHELL_PROFILE"; then
        echo "" >> "$SHELL_PROFILE"
        echo "# vcpkg configuration" >> "$SHELL_PROFILE"
        echo "export VCPKG_ROOT=\"$VCPKG_DIR\"" >> "$SHELL_PROFILE"
        echo "export PATH=\"\$VCPKG_ROOT:\$PATH\"" >> "$SHELL_PROFILE"
        print_success "Added vcpkg to $SHELL_PROFILE"
    else
        print_warning "vcpkg environment already configured in $SHELL_PROFILE"
    fi
fi

# 安装基本包
print_status "Installing basic packages..."
cd "$VCPKG_DIR"

# 检测平台
OS="$(uname)"
ARCH="$(uname -m)"

case "$OS" in
    "Darwin")
        case "$ARCH" in
            "x86_64")
                TRIPLET="x64-osx"
                ;;
            "arm64")
                TRIPLET="arm64-osx"
                ;;
            *)
                TRIPLET="x64-osx"
                ;;
        esac
        ;;
    "Linux")
        case "$ARCH" in
            "x86_64")
                TRIPLET="x64-linux"
                ;;
            "aarch64"|"arm64")
                TRIPLET="arm64-linux"
                ;;
            *)
                TRIPLET="x64-linux"
                ;;
        esac
        ;;
    *)
        print_warning "Unknown OS: $OS, using x64-linux"
        TRIPLET="x64-linux"
        ;;
esac

print_status "Installing packages for $TRIPLET..."

# 基本包列表
PACKAGES="openssl sqlite3 zlib"

for package in $PACKAGES; do
    print_status "Installing $package..."
    if ./vcpkg install "$package:$TRIPLET"; then
        print_success "$package installed successfully"
    else
        print_warning "Failed to install $package, continuing..."
    fi
done

print_success "vcpkg setup completed!"
print_status "VCPKG_ROOT: $VCPKG_ROOT"
print_status "Please restart your shell or run: source $SHELL_PROFILE"
print_status "Then you can use: ./scripts/build-vcpkg-only.sh"
