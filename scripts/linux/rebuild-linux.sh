#!/bin/bash

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 获取脚本所在目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
APP_DIR="$PROJECT_ROOT/app"
ENV_FILE="$SCRIPT_DIR/../.env"

# 解析命令行参数
RUST_TARGET="x86_64-unknown-linux-gnu"

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            RUST_TARGET="$2"
            shift 2
            ;;
        -h|--help)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --target TARGET    指定 Rust 编译目标"
            echo "                     默认: x86_64-unknown-linux-gnu"
            echo "                     支持: x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl,"
            echo "                          aarch64-unknown-linux-gnu, armv7-unknown-linux-gnueabihf"
            echo "  -h, --help         显示此帮助信息"
            exit 0
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            exit 1
            ;;
    esac
done

# 验证 target
SUPPORTED_TARGETS=("x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl" "aarch64-unknown-linux-gnu" "armv7-unknown-linux-gnueabihf")
if [[ ! " ${SUPPORTED_TARGETS[@]} " =~ " ${RUST_TARGET} " ]]; then
    echo -e "${RED}错误: 不支持的 target '$RUST_TARGET'${NC}"
    exit 1
fi

# 加载 .env 文件
if [ -f "$ENV_FILE" ]; then
    echo -e "${GREEN}✓ 加载环境变量...${NC}"
    
    while IFS= read -r line || [ -n "$line" ]; do
        line=$(echo "$line" | tr -d '\r')
        if [[ -z "$line" ]] || [[ "$line" =~ ^[[:space:]]*# ]]; then
            continue
        fi
        if [[ "$line" =~ ^([^=]+)=(.*)$ ]]; then
            key="${BASH_REMATCH[1]}"
            value="${BASH_REMATCH[2]}"
            key=$(echo "$key" | xargs)
            value=$(echo "$value" | xargs)
            value="${value%\"}"
            value="${value#\"}"
            value="${value%\'}"
            value="${value#\'}"
            export "$key=$value"
        fi
    done < "$ENV_FILE"
else
    echo -e "${RED}错误: 未找到 .env 文件${NC}"
    exit 1
fi

# 检查环境变量
if [ -z "$TAURI_PRIVATE_KEY" ] || [ -z "$TAURI_KEY_PASSWORD" ]; then
    echo -e "${RED}错误: 请在 scripts/.env 文件中设置环境变量${NC}"
    exit 1
fi

# 设置交叉编译环境变量
CROSS_COMPILE_ENV=""
case "$RUST_TARGET" in
    x86_64-unknown-linux-gnu)
        # x86_64 原生编译，不需要特殊配置
        CROSS_COMPILE_ENV=""
        ;;
    x86_64-unknown-linux-musl)
        CROSS_COMPILE_ENV="
        export PKG_CONFIG_ALLOW_CROSS=1
        export CC=musl-gcc
        export CXX=musl-g++
        export AR=ar
        export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
        export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig
        export PKG_CONFIG_LIBDIR=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig
        "
        ;;
    aarch64-unknown-linux-gnu)
        CROSS_COMPILE_ENV="
        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig
        export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig
        export PKG_CONFIG_SYSROOT_DIR=/
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
        export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
        export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
        export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
        "
        ;;
    armv7-unknown-linux-gnueabihf)
        CROSS_COMPILE_ENV="
        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig:/usr/share/pkgconfig
        export PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf/pkgconfig:/usr/share/pkgconfig
        export PKG_CONFIG_SYSROOT_DIR=/
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
        export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
        export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
        export AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar
        "
        ;;
esac

echo -e "${GREEN}✓ 环境变量验证通过${NC}"
echo -e "${YELLOW}开始快速构建 (target: $RUST_TARGET)...${NC}"

docker run --rm \
  -v "$APP_DIR:/app" \
  -e CARGO_HOME=/app/.cargo-cache \
  -e CI=true \
  -e TAURI_PRIVATE_KEY="$TAURI_PRIVATE_KEY" \
  -e TAURI_KEY_PASSWORD="$TAURI_KEY_PASSWORD" \
  etcd-workbench-linux-builder \
  bash -c "
    $CROSS_COMPILE_ENV
    rustup target add $RUST_TARGET
    pnpm install --frozen-lockfile
    pnpm tauri build --target $RUST_TARGET
  "

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ 构建成功！${NC}"
    echo -e "输出文件："
    echo -e "  DEB: app/src-tauri/target/$RUST_TARGET/release/bundle/deb/"
    echo -e "  AppImage: app/src-tauri/target/$RUST_TARGET/release/bundle/appimage/"
else
    echo -e "${RED}✗ 构建失败${NC}"
    exit 1
fi