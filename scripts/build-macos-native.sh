#!/bin/bash
# Native macOS build script that avoids all GTK dependencies

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

BUILD_MODE="${1:-release}"

print_status "Building SeeU Desktop for macOS (native)"
print_status "Build mode: $BUILD_MODE"

# 检查是否在 macOS 上
if [ "$(uname)" != "Darwin" ]; then
    print_error "This script must be run on macOS"
    exit 1
fi

# 设置环境变量来避免 pkg-config 和 GTK
export PKG_CONFIG_ALLOW_CROSS=0
export PKG_CONFIG=false
export OPENSSL_STATIC=1
export OPENSSL_VENDORED=1
export LIBSQLITE3_SYS_USE_PKG_CONFIG=0
export LIBZ_SYS_STATIC=1
export SYSTEM_DEPS_NO_PKG_CONFIG=1

# 禁用所有可能的 pkg-config 查找
NO_PKG_CONFIG_LIBS=(
    "GLIB_2_0" "GOBJECT_2_0" "GIO_2_0" "CAIRO" "PANGO" "PANGOCAIRO"
    "GDK_PIXBUF_2_0" "GTK_3_0" "ATK" "FONTCONFIG" "FREETYPE2" "HARFBUZZ"
    "EXPAT" "BZIP2" "ZLIB" "LIBPNG" "JPEG"
)

for lib in "${NO_PKG_CONFIG_LIBS[@]}"; do
    export "${lib}_NO_PKG_CONFIG=1"
done

# 创建输出目录
mkdir -p dist/macos

# 备份原始 Cargo.toml
cp Cargo.toml Cargo.toml.backup

# 直接修改原始 Cargo.toml，只移除 rfd 依赖
cp Cargo.toml.backup Cargo.toml.macos
# 移除 rfd 依赖行
sed -i '' '/^rfd = /d' Cargo.toml.macos

# 同样需要更新子 crate 的 Cargo.toml 来移除 rfd 依赖
for crate_dir in crates/*/; do
    if [ -f "$crate_dir/Cargo.toml" ]; then
        print_status "Updating $crate_dir/Cargo.toml to remove GTK dependencies..."
        
        # 备份
        cp "$crate_dir/Cargo.toml" "$crate_dir/Cargo.toml.backup"
        
        # 移除 rfd 依赖行
        sed '/^rfd = /d' "$crate_dir/Cargo.toml.backup" > "$crate_dir/Cargo.toml"
        
        # 确保 egui/eframe 使用正确的特性
        sed -i '' 's/eframe = "0.24.1"/eframe = { version = "0.24.1", default-features = false, features = ["default_fonts", "glow"] }/g' "$crate_dir/Cargo.toml"
        sed -i '' 's/egui = "0.24.1"/egui = { version = "0.24.1", default-features = false, features = ["default_fonts"] }/g' "$crate_dir/Cargo.toml"
    fi
done

# 构建参数
BUILD_FLAGS=""
if [ "$BUILD_MODE" = "release" ]; then
    BUILD_FLAGS="--release"
fi

# 尝试构建
print_status "Building with macOS-optimized configuration..."
if CARGO_MANIFEST_PATH="Cargo.toml.macos" cargo build $BUILD_FLAGS; then
    print_success "Build completed successfully!"
    
    # 确定二进制路径
    BINARY_PATH="target"
    if [ "$BUILD_MODE" = "release" ]; then
        BINARY_PATH="$BINARY_PATH/release"
    else
        BINARY_PATH="$BINARY_PATH/debug"
    fi
    
    # 复制二进制文件
    if [ -f "$BINARY_PATH/seeu_desktop" ]; then
        cp "$BINARY_PATH/seeu_desktop" "dist/macos/"
        print_success "Binary copied to dist/macos/seeu_desktop"
        
        # 显示二进制信息
        file "dist/macos/seeu_desktop"
        size=$(du -h "dist/macos/seeu_desktop" | cut -f1)
        print_status "Binary size: $size"
        
        # 检查依赖
        print_status "Binary dependencies:"
        otool -L "dist/macos/seeu_desktop" | grep -v "dist/macos/seeu_desktop:"
        
        # 复制资源文件
        if [ -d "assets" ]; then
            cp -r assets "dist/macos/"
            print_status "Assets copied to dist/macos/"
        fi
        
        # 创建启动脚本
        cat > "dist/macos/seeu-desktop.sh" << 'LAUNCHER_EOF'
#!/bin/bash
cd "$(dirname "$0")"
./seeu_desktop "$@"
LAUNCHER_EOF
        chmod +x "dist/macos/seeu-desktop.sh"
        print_status "Launcher script created"
        
        # 创建 macOS 应用程序包
        create_app_bundle() {
            local app_name="SeeU Desktop"
            local bundle_dir="dist/macos/$app_name.app"
            
            print_status "Creating macOS app bundle..."
            
            # 创建包结构
            mkdir -p "$bundle_dir/Contents/MacOS"
            mkdir -p "$bundle_dir/Contents/Resources"
            
            # 复制二进制文件
            cp "dist/macos/seeu_desktop" "$bundle_dir/Contents/MacOS/"
            
            # 复制资源
            if [ -d "assets" ]; then
                cp -r assets/* "$bundle_dir/Contents/Resources/"
            fi
            
            # 创建 Info.plist
            cat > "$bundle_dir/Contents/Info.plist" << PLIST_EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>seeu_desktop</string>
    <key>CFBundleIdentifier</key>
    <string>com.seeu.desktop</string>
    <key>CFBundleName</key>
    <string>SeeU Desktop</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST_EOF
            
            print_success "App bundle created: $bundle_dir"
        }
        
        if [ "$BUILD_MODE" = "release" ]; then
            create_app_bundle
        fi
        
    else
        print_error "Binary not found at $BINARY_PATH/seeu_desktop"
        # 恢复原始配置
        mv Cargo.toml.backup Cargo.toml
        for crate_dir in crates/*/; do
            if [ -f "$crate_dir/Cargo.toml.backup" ]; then
                mv "$crate_dir/Cargo.toml.backup" "$crate_dir/Cargo.toml"
            fi
        done
        rm -f Cargo.toml.macos
        exit 1
    fi
    
else
    print_error "Build failed with macOS-optimized configuration"
    
    # 恢复原始配置
    mv Cargo.toml.backup Cargo.toml
    for crate_dir in crates/*/; do
        if [ -f "$crate_dir/Cargo.toml.backup" ]; then
            mv "$crate_dir/Cargo.toml.backup" "$crate_dir/Cargo.toml"
        fi
    done
    rm -f Cargo.toml.macos
    exit 1
fi

# 清理
print_status "Cleaning up..."
mv Cargo.toml.backup Cargo.toml
for crate_dir in crates/*/; do
    if [ -f "$crate_dir/Cargo.toml.backup" ]; then
        mv "$crate_dir/Cargo.toml.backup" "$crate_dir/Cargo.toml"
    fi
done
rm -f Cargo.toml.macos

print_success "macOS native build completed successfully!"
print_status "Binary available at: dist/macos/seeu_desktop"
print_status "Run with: ./dist/macos/seeu-desktop.sh"
if [ "$BUILD_MODE" = "release" ]; then
    print_status "App bundle available at: dist/macos/SeeU Desktop.app"
fi
