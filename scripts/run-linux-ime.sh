#!/bin/bash

# Linux IME 支持启动脚本
# 用于在Linux下启动SeeU Desktop时设置正确的IME环境变量

# 设置IME相关环境变量
export GTK_IM_MODULE=ibus
export QT_IM_MODULE=ibus
export XMODIFIERS=@im=ibus

# 如果使用fcitx输入法，取消注释以下行
# export GTK_IM_MODULE=fcitx
# export QT_IM_MODULE=fcitx
# export XMODIFIERS=@im=fcitx

# 如果使用scim输入法，取消注释以下行
# export GTK_IM_MODULE=scim
# export QT_IM_MODULE=scim
# export XMODIFIERS=@im=scim

# 设置字体相关环境变量
export FONTCONFIG_PATH=/etc/fonts

# 启动应用程序
echo "启动 SeeU Desktop with IME support..."
echo "IME Module: $GTK_IM_MODULE"
echo "Qt IM Module: $QT_IM_MODULE"
echo "XModifiers: $XMODIFIERS"

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# 切换到项目目录
cd "$PROJECT_DIR"

# 检查是否存在编译好的二进制文件
if [ -f "target/release/seeu_desktop" ]; then
    echo "运行 release 版本..."
    ./target/release/seeu_desktop "$@"
elif [ -f "target/debug/seeu_desktop" ]; then
    echo "运行 debug 版本..."
    ./target/debug/seeu_desktop "$@"
else
    echo "未找到编译好的二进制文件，尝试编译并运行..."
    cargo run --release "$@"
fi
