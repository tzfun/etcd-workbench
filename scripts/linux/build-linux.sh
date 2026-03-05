#!/bin/bash

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Etcd Workbench Linux Builder${NC}"
echo -e "${GREEN}========================================${NC}"

# 获取脚本所在目录（scripts/linux）
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# 项目根目录
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
# app 目录
APP_DIR="$PROJECT_ROOT/app"
# .env 文件路径
ENV_FILE="$SCRIPT_DIR/../.env"

# 解析命令行参数
RUST_TARGET="x86_64-unknown-linux-gnu"  # Linux 默认 target
SKIP_CLEAN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            RUST_TARGET="$2"
            shift 2
            ;;
        --skip-clean)
            SKIP_CLEAN=true
            shift
            ;;
        -h|--help)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --target TARGET    指定 Rust 编译目标"
            echo "                     默认: x86_64-unknown-linux-gnu"
            echo "                     支持的目标:"
            echo "                       - x86_64-unknown-linux-gnu (Linux x64 glibc, 默认)"
            echo "                       - x86_64-unknown-linux-musl (Linux x64 musl, 静态链接)"
            echo "                       - aarch64-unknown-linux-gnu (Linux ARM64)"
            echo "                       - armv7-unknown-linux-gnueabihf (Linux ARMv7)"
            echo "  --skip-clean       跳过清理构建文件"
            echo "  -h, --help         显示此帮助信息"
            echo ""
            echo "说明:"
            echo "  - glibc 版本: 标准 Linux 版本，动态链接"
            echo "  - musl 版本: 静态链接，更好的可移植性，二进制文件更大"
            echo ""
            echo "示例:"
            echo "  $0                                          # 构建 x86_64 glibc"
            echo "  $0 --target x86_64-unknown-linux-musl       # 构建 x86_64 musl"
            echo "  $0 --target aarch64-unknown-linux-gnu       # 构建 ARM64"
            echo "  $0 --target armv7-unknown-linux-gnueabihf   # 构建 ARMv7"
            echo "  $0 --skip-clean                             # 构建且不清理"
            exit 0
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            echo "使用 --help 查看帮助信息"
            exit 1
            ;;
    esac
done

# 验证 target 是否支持
SUPPORTED_TARGETS=("x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl" "aarch64-unknown-linux-gnu" "armv7-unknown-linux-gnueabihf")
if [[ ! " ${SUPPORTED_TARGETS[@]} " =~ " ${RUST_TARGET} " ]]; then
    echo -e "${RED}错误: 不支持的 target '$RUST_TARGET'${NC}"
    echo "支持的 targets: ${SUPPORTED_TARGETS[@]}"
    exit 1
fi

echo -e "${YELLOW}项目根目录: $PROJECT_ROOT${NC}"
echo -e "${YELLOW}App 目录: $APP_DIR${NC}"
echo -e "${YELLOW}Rust Target: $RUST_TARGET${NC}"

# 检查 Docker 是否运行
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}错误: Docker 未运行，请先启动 Docker Desktop${NC}"
    exit 1
fi

# 检查 app 目录是否存在
if [ ! -d "$APP_DIR" ]; then
    echo -e "${RED}错误: 未找到 app 目录${NC}"
    exit 1
fi

# 检查是否在 app 目录中有 Tauri 项目
if [ ! -f "$APP_DIR/package.json" ] || [ ! -d "$APP_DIR/src-tauri" ]; then
    echo -e "${RED}错误: app 目录中未找到有效的 Tauri 项目${NC}"
    exit 1
fi

# 检查是否使用 pnpm
if [ ! -f "$APP_DIR/pnpm-lock.yaml" ]; then
    echo -e "${YELLOW}警告: 未找到 pnpm-lock.yaml，确认项目使用 pnpm？${NC}"
    read -p "继续? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 加载 .env 文件
if [ -f "$ENV_FILE" ]; then
    echo -e "${GREEN}✓ 找到 .env 文件，正在加载环境变量...${NC}"
    
    # 读取并处理 .env 文件
    while IFS= read -r line || [ -n "$line" ]; do
        # 移除可能的 \r (Windows 换行符)
        line=$(echo "$line" | tr -d '\r')
        
        # 跳过空行和注释
        if [[ -z "$line" ]] || [[ "$line" =~ ^[[:space:]]*# ]]; then
            continue
        fi
        
        # 提取 key 和 value
        if [[ "$line" =~ ^([^=]+)=(.*)$ ]]; then
            key="${BASH_REMATCH[1]}"
            value="${BASH_REMATCH[2]}"
            
            # 去除 key 两端空格
            key=$(echo "$key" | xargs)
            
            # 去除 value 两端的引号和空格
            value=$(echo "$value" | xargs)
            value="${value%\"}"
            value="${value#\"}"
            value="${value%\'}"
            value="${value#\'}"
            
            # 导出环境变量
            export "$key=$value"
            
            # 显示（隐藏敏感信息）
            if [[ "$key" == "TAURI_PRIVATE_KEY" || "$key" == "TAURI_KEY_PASSWORD" ]]; then
                echo -e "  ${GREEN}✓${NC} $key: [已设置，长度: ${#value}]"
            else
                echo -e "  ${GREEN}✓${NC} $key=$value"
            fi
        fi
    done < "$ENV_FILE"
    
    echo ""
else
    echo -e "${YELLOW}警告: 未找到 .env 文件 ($ENV_FILE)${NC}"
fi

# 检查必需的环境变量
if [ -z "$TAURI_PRIVATE_KEY" ]; then
    echo -e "${RED}错误: 未设置 TAURI_PRIVATE_KEY 环境变量${NC}"
    echo -e "${YELLOW}请在 scripts/.env 文件中设置：${NC}"
    echo -e "  TAURI_PRIVATE_KEY=\"your-private-key\""
    echo -e "  TAURI_KEY_PASSWORD=\"your-key-password\""
    exit 1
fi

if [ -z "$TAURI_KEY_PASSWORD" ]; then
    echo -e "${RED}错误: 未设置 TAURI_KEY_PASSWORD 环境变量${NC}"
    echo -e "${YELLOW}请在 scripts/.env 文件中设置：${NC}"
    echo -e "  TAURI_PRIVATE_KEY=\"your-private-key\""
    echo -e "  TAURI_KEY_PASSWORD=\"your-key-password\""
    exit 1
fi

echo -e "${GREEN}✓ 环境变量检查通过${NC}"

# 构建 Docker 镜像
echo -e "${YELLOW}正在构建 Docker 镜像...${NC}"
cd "$SCRIPT_DIR"
docker build --no-cache -t etcd-workbench-linux-builder .

if [ $? -ne 0 ]; then
    echo -e "${RED}Docker 镜像构建失败${NC}"
    exit 1
fi

echo -e "${GREEN}Docker 镜像构建成功${NC}"

# 清理之前的构建（可选）
if [ "$SKIP_CLEAN" = false ]; then
    read -p "是否清理之前的构建文件? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}清理中...${NC}"
        rm -rf "$APP_DIR/src-tauri/target/$RUST_TARGET"
    fi
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

# 运行构建
echo -e "${YELLOW}开始构建 Linux 包 (target: $RUST_TARGET)...${NC}"
echo -e "${YELLOW}这可能需要几分钟时间，请耐心等待...${NC}"

docker run --rm \
  -v "$APP_DIR:/app" \
  -e CARGO_HOME=/app/.cargo-cache \
  -e CI=true \
  -e TAURI_PRIVATE_KEY="$TAURI_PRIVATE_KEY" \
  -e TAURI_KEY_PASSWORD="$TAURI_KEY_PASSWORD" \
  etcd-workbench-linux-builder \
  bash -c "
    set -e
    
    echo '=== 环境信息 ==='
    echo 'Node 版本: '\$(node --version)
    echo 'npm 版本: '\$(npm --version)
    echo 'pnpm 版本: '\$(pnpm --version)
    echo 'Rust 版本: '\$(rustc --version)
    echo 'Protoc 版本: '\$(protoc --version)
    echo '工作目录: '\$(pwd)
    echo 'TAURI_PRIVATE_KEY: [已设置]'
    echo 'TAURI_KEY_PASSWORD: [已设置]'
    echo 'Rust Target: $RUST_TARGET'
    echo ''
    
    $CROSS_COMPILE_ENV
    
    echo '=== 添加 Rust target ==='
    rustup target add $RUST_TARGET
    
    echo ''
    echo '=== 安装 pnpm 依赖 ==='
    pnpm install --frozen-lockfile
    
    echo ''
    echo '=== 开始 Tauri 构建 ==='
    pnpm tauri build --target $RUST_TARGET
    
    echo ''
    echo '=== 构建完成 ==='
    echo '输出文件位置：'
    echo '  Target 目录: src-tauri/target/$RUST_TARGET/release/'
    echo '  Bundle 目录: src-tauri/target/$RUST_TARGET/release/bundle/'
  "

if [ $? -eq 0 ]; then
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}构建成功！${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo -e "输出文件位置："
    echo -e "  ${GREEN}DEB 包:${NC} app/src-tauri/target/$RUST_TARGET/release/bundle/deb/"
    echo -e "  ${GREEN}AppImage:${NC} app/src-tauri/target/$RUST_TARGET/release/bundle/appimage/"
    
    # 列出生成的文件
    echo -e "\n${YELLOW}生成的文件：${NC}"
    ls -lh "$APP_DIR/src-tauri/target/$RUST_TARGET/release/bundle/deb/"*.deb 2>/dev/null || echo "  未找到 .deb 文件"
    ls -lh "$APP_DIR/src-tauri/target/$RUST_TARGET/release/bundle/appimage/"*.AppImage 2>/dev/null || echo "  未找到 .AppImage 文件"
else
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}构建失败，请查看上面的错误信息${NC}"
    echo -e "${RED}========================================${NC}"
    exit 1
fi