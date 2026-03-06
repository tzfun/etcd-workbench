#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
APP_DIR="$PROJECT_ROOT/app"
BIN_DIR="$PROJECT_ROOT/bin/app"
ENV_FILE="$SCRIPT_DIR/../.env"

# 解析参数
ARCH="x86_64"
ACTION="build"

while [[ $# -gt 0 ]]; do
    case $1 in
        --arch)
            ARCH="$2"
            shift 2
            ;;
        --rebuild)
            ACTION="rebuild"
            shift
            ;;
        -h|--help)
            echo "用法: $0 [选项]"
            echo ""
            echo "选项:"
            echo "  --arch ARCH     指定架构: x86_64, aarch64, all"
            echo "                  默认: x86_64"
            echo "  --rebuild       快速重建（不重建镜像）"
            echo "  -h, --help      显示帮助"
            echo ""
            echo "示例:"
            echo "  $0                      # 构建 x86_64"
            echo "  $0 --arch aarch64       # 构建 ARM64"
            echo "  $0 --arch all           # 构建所有架构"
            echo "  $0 --rebuild            # 快速重建 x86_64"
            exit 0
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            exit 1
            ;;
    esac
done

# 验证架构
VALID_ARCHS=("x86_64" "aarch64" "all")
if [[ ! " ${VALID_ARCHS[@]} " =~ " ${ARCH} " ]]; then
    echo -e "${RED}错误: 不支持的架构 '$ARCH'${NC}"
    echo "支持的架构: ${VALID_ARCHS[@]}"
    exit 1
fi

# 检查环境
if [ ! -d "$APP_DIR" ]; then
    echo -e "${RED}错误: 未找到 app 目录${NC}"
    exit 1
fi

# 加载环境变量
if [ -f "$ENV_FILE" ]; then
    echo -e "${GREEN}✓ 加载环境变量...${NC}"
    while IFS= read -r line || [ -n "$line" ]; do
        line=$(echo "$line" | tr -d '\r')
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
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
fi

if [ -z "$TAURI_PRIVATE_KEY" ] || [ -z "$TAURI_KEY_PASSWORD" ]; then
    echo -e "${RED}错误: 请设置环境变量${NC}"
    exit 1
fi

# 移动构建产物到 bin 目录
move_artifacts() {
    local arch=$1
    local rust_target=$2
    
    local target_dir="$APP_DIR/src-tauri/target/${rust_target}/release/bundle"
    local bin_subdir
    local arch_name
    
    # 确定 bin 子目录名称和架构名称
    case "$arch" in
        x86_64)
            bin_subdir="linux-x86_64"
            arch_name="x86_64"
            ;;
        aarch64)
            bin_subdir="linux-aarch64"
            arch_name="arm64"
            ;;
    esac
    
    local dest_dir="$BIN_DIR/$bin_subdir"
    
    # 创建目标目录
    mkdir -p "$dest_dir"
    
    echo -e "${YELLOW}移动构建产物到 bin 目录...${NC}"
    
    # 移动 DEB 文件
    if ls "$target_dir/deb/"*.deb 1> /dev/null 2>&1; then
        for deb_file in "$target_dir/deb/"*.deb; do
            # 提取版本号（从原文件名中提取）
            local original_filename=$(basename "$deb_file")
            local version=$(echo "$original_filename" | sed -E 's/etcd-workbench_([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
            
            # 新文件名格式: etcd-workbench-{版本号}-linux-{架构}.deb
            local new_filename="etcd-workbench-${version}-linux-${arch_name}.deb"
            
            cp "$deb_file" "$dest_dir/$new_filename"
            echo -e "  ${GREEN}✓${NC} DEB: $dest_dir/$new_filename"
        done
    fi
    
    # 移动 AppImage 文件
    if ls "$target_dir/appimage/"*.AppImage 1> /dev/null 2>&1; then
        for appimage_file in "$target_dir/appimage/"*.AppImage; do
            # 提取版本号（从原文件名中提取）
            local original_filename=$(basename "$appimage_file")
            local version=$(echo "$original_filename" | sed -E 's/etcd-workbench_([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
            
            # 新文件名格式: etcd-workbench-{版本号}-linux-{架构}.AppImage
            local new_filename="etcd-workbench-${version}-linux-${arch_name}.AppImage"
            
            cp "$appimage_file" "$dest_dir/$new_filename"
            echo -e "  ${GREEN}✓${NC} AppImage: $dest_dir/$new_filename"
        done
    fi
    
    echo -e "${GREEN}✓ 构建产物已移动到: $dest_dir${NC}"
}

# 构建函数
build_arch() {
    local arch=$1
    local image_name="etcd-workbench-linux-${arch}"
    local dockerfile="${SCRIPT_DIR}/${arch}/Dockerfile"
    
    # 定义各架构的 Rust target
    local rust_target
    case "$arch" in
        x86_64)
            rust_target="x86_64-unknown-linux-gnu"
            ;;
        aarch64)
            rust_target="aarch64-unknown-linux-gnu"
            ;;
    esac
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}构建 ${arch}${NC}"
    echo -e "${GREEN}Target: ${rust_target}${NC}"
    echo -e "${GREEN}========================================${NC}"
    
    if [ "$ACTION" = "build" ]; then
        echo -e "${YELLOW}构建 Docker 镜像...${NC}"
        
        # 根据架构选择平台
        local platform
        case "$arch" in
            x86_64)
                platform="linux/amd64"
                ;;
            aarch64)
                platform="linux/arm64"
                echo -e "${YELLOW}注意: ARM64 构建在非 ARM64 主机上会使用模拟，可能较慢${NC}"
                ;;
        esac
        
        docker build --platform "$platform" -f "$dockerfile" -t "$image_name" "$SCRIPT_DIR/$arch"
        if [ $? -ne 0 ]; then
            echo -e "${RED}Docker 镜像构建失败${NC}"
            return 1
        fi
        echo -e "${GREEN}✓ Docker 镜像构建成功${NC}"
    fi
    
    echo -e "${YELLOW}开始构建 ${arch} 包...${NC}"
    
    # 根据架构选择平台
    local platform
    case "$arch" in
        x86_64)
            platform="linux/amd64"
            ;;
        aarch64)
            platform="linux/arm64"
            ;;
    esac
    
    docker run --rm \
        --platform "$platform" \
        -v "$APP_DIR:/app" \
        -e CI=true \
        -e TAURI_PRIVATE_KEY="$TAURI_PRIVATE_KEY" \
        -e TAURI_KEY_PASSWORD="$TAURI_KEY_PASSWORD" \
        "$image_name" \
        bash -c "
            echo '=== 环境信息 ==='
            node --version
            pnpm --version
            rustc --version
            uname -m
            echo 'Target: $rust_target'
            echo ''
            
            echo '=== 添加 Rust target ==='
            rustup target add $rust_target
            
            echo ''
            echo '=== 安装依赖 ==='
            pnpm install --frozen-lockfile
            
            echo ''
            echo '=== 开始构建 ==='
            pnpm tauri build --target $rust_target
            
            echo ''
            echo '=== 构建完成 ==='
            echo '输出目录: src-tauri/target/$rust_target/release/bundle/'
        "
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}========================================${NC}"
        echo -e "${GREEN}✓ ${arch} 构建成功！${NC}"
        echo -e "${GREEN}========================================${NC}"
        echo -e "输出文件："
        echo -e "  ${GREEN}DEB:${NC} app/src-tauri/target/${rust_target}/release/bundle/deb/"
        echo -e "  ${GREEN}AppImage:${NC} app/src-tauri/target/${rust_target}/release/bundle/appimage/"
        echo ""
        echo -e "${YELLOW}生成的文件：${NC}"
        ls -lh "$APP_DIR/src-tauri/target/${rust_target}/release/bundle/deb/"*.deb 2>/dev/null || echo "  未找到 .deb 文件"
        ls -lh "$APP_DIR/src-tauri/target/${rust_target}/release/bundle/appimage/"*.AppImage 2>/dev/null || echo "  未找到 .AppImage 文件"
        
        echo ""
        # 移动构建产物
        move_artifacts "$arch" "$rust_target"
        
        return 0
    else
        echo -e "${RED}✗ ${arch} 构建失败${NC}"
        return 1
    fi
}

# 执行构建
if [ "$ARCH" = "all" ]; then
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}构建所有架构${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    
    FAILED_ARCHS=()
    
    for arch in x86_64 aarch64; do
        echo ""
        build_arch "$arch"
        if [ $? -ne 0 ]; then
            FAILED_ARCHS+=("$arch")
        fi
        echo ""
    done
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}构建总结${NC}"
    echo -e "${GREEN}========================================${NC}"
    
    if [ ${#FAILED_ARCHS[@]} -eq 0 ]; then
        echo -e "${GREEN}✓ 所有架构构建成功！${NC}"
        echo ""
        echo -e "${YELLOW}构建产物位置：${NC}"
        echo -e "  ${GREEN}x86_64:${NC} bin/app/linux-x86_64/"
        echo -e "  ${GREEN}aarch64:${NC} bin/app/linux-aarch64/"
        echo ""
        echo -e "${YELLOW}文件列表：${NC}"
        ls -lh "$BIN_DIR/linux-x86_64/" 2>/dev/null
        echo ""
        ls -lh "$BIN_DIR/linux-aarch64/" 2>/dev/null
    else
        echo -e "${RED}以下架构构建失败：${NC}"
        for arch in "${FAILED_ARCHS[@]}"; do
            echo -e "  ${RED}✗${NC} $arch"
        done
        exit 1
    fi
else
    build_arch "$ARCH"
fi