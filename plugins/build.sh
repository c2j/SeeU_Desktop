#!/bin/bash

# SeeU Desktop Plugin Build Script
# 构建所有示例插件为 WASM 模块

set -e

echo "🚀 Building SeeU Desktop Plugins..."

# 检查 Rust 和 WASM 目标是否安装
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first."
    exit 1
fi

if ! rustup target list --installed | grep -q "wasm32-wasi"; then
    echo "📦 Installing wasm32-wasi target..."
    rustup target add wasm32-wasi
fi

# 创建输出目录
OUTPUT_DIR="dist"
mkdir -p "$OUTPUT_DIR"

echo "📁 Output directory: $OUTPUT_DIR"

# 构建函数
build_plugin() {
    local plugin_name=$1
    local plugin_dir="examples/$plugin_name"
    
    if [ ! -d "$plugin_dir" ]; then
        echo "⚠️  Plugin directory not found: $plugin_dir"
        return 1
    fi
    
    echo "🔨 Building plugin: $plugin_name"
    
    cd "$plugin_dir"
    
    # 构建 WASM 模块
    cargo build --target wasm32-wasi --release
    
    if [ $? -eq 0 ]; then
        # 复制 WASM 文件和清单到输出目录
        local wasm_file="target/wasm32-wasi/release/${plugin_name//-/_}_plugin.wasm"
        local output_plugin_dir="../../$OUTPUT_DIR/$plugin_name"
        
        mkdir -p "$output_plugin_dir"
        
        if [ -f "$wasm_file" ]; then
            cp "$wasm_file" "$output_plugin_dir/"
            echo "✅ WASM file copied: $wasm_file"
        else
            echo "⚠️  WASM file not found: $wasm_file"
        fi
        
        if [ -f "plugin.json" ]; then
            cp "plugin.json" "$output_plugin_dir/"
            echo "✅ Plugin manifest copied"
        else
            echo "⚠️  Plugin manifest not found: plugin.json"
        fi
        
        if [ -f "README.md" ]; then
            cp "README.md" "$output_plugin_dir/"
            echo "✅ README copied"
        fi
        
        echo "✅ Plugin $plugin_name built successfully"
    else
        echo "❌ Failed to build plugin: $plugin_name"
        return 1
    fi
    
    cd - > /dev/null
}

# 构建 SDK
echo "🔧 Building Plugin SDK..."
cd "sdk/rust"
cargo build --release
if [ $? -eq 0 ]; then
    echo "✅ Plugin SDK built successfully"
else
    echo "❌ Failed to build Plugin SDK"
    exit 1
fi
cd - > /dev/null

# 构建所有示例插件
PLUGINS=("hello-world" "file-tools" "ai-prompts")

for plugin in "${PLUGINS[@]}"; do
    build_plugin "$plugin"
    echo ""
done

# 创建插件包索引
echo "📋 Creating plugin index..."
cat > "$OUTPUT_DIR/index.json" << EOF
{
  "version": "1.0",
  "plugins": [
    {
      "name": "hello-world",
      "display_name": "Hello World Plugin",
      "version": "0.1.0",
      "description": "一个简单的 Hello World 插件，演示基本功能",
      "author": "SeeU Team",
      "categories": ["tools", "examples"],
      "entry_point": "hello_world_plugin.wasm",
      "manifest": "plugin.json"
    },
    {
      "name": "file-tools",
      "display_name": "File Tools Plugin",
      "version": "0.1.0",
      "description": "文件操作工具集合，提供文件列表、搜索、信息查看等功能",
      "author": "SeeU Team",
      "categories": ["tools", "filesystem"],
      "entry_point": "file_tools_plugin.wasm",
      "manifest": "plugin.json"
    },
    {
      "name": "ai-prompts",
      "display_name": "AI Prompts Plugin",
      "version": "0.1.0",
      "description": "AI提示模板集合，提供代码审查、解释、文档编写、调试帮助等专业模板",
      "author": "SeeU Team",
      "categories": ["prompts", "ai", "development"],
      "entry_point": "ai_prompts_plugin.wasm",
      "manifest": "plugin.json"
    }
  ]
}
EOF

echo "✅ Plugin index created: $OUTPUT_DIR/index.json"

# 显示构建结果
echo ""
echo "🎉 Build completed!"
echo "📦 Built plugins:"
for plugin in "${PLUGINS[@]}"; do
    if [ -d "$OUTPUT_DIR/$plugin" ]; then
        echo "  ✅ $plugin"
    else
        echo "  ❌ $plugin"
    fi
done

echo ""
echo "📁 Output directory structure:"
find "$OUTPUT_DIR" -type f | sort

echo ""
echo "🚀 Plugins are ready for installation in SeeU Desktop!"
echo "💡 Copy the plugin directories from '$OUTPUT_DIR' to your SeeU Desktop plugins folder."
