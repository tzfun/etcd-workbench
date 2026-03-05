# 颜色输出
$RED = "`e[0;31m"
$GREEN = "`e[0;32m"
$YELLOW = "`e[1;33m"
$NC = "`e[0m"

Write-Host "${GREEN}========================================${NC}"
Write-Host "${GREEN}Etcd Workbench Linux Builder${NC}"
Write-Host "${GREEN}========================================${NC}"

# 获取脚本所在目录（scripts/windows）
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
# 项目根目录
$PROJECT_ROOT = Resolve-Path "$SCRIPT_DIR\..\.."
# app 目录
$APP_DIR = "$PROJECT_ROOT\app"
# .env 文件路径
$ENV_FILE = "$SCRIPT_DIR\..\.env"

# 解析命令行参数
$RUST_TARGET = "x86_64-unknown-linux-gnu"  # Linux 默认 target
$SKIP_CLEAN = $false

$argsList = $args
while ($argsList.Count -gt 0) {
    switch ($argsList[0]) {
        "--target" {
            $RUST_TARGET = $argsList[1]
            $argsList = $argsList[2..($argsList.Count-1)]
        }
        "--skip-clean" {
            $SKIP_CLEAN = $true
            $argsList = $argsList[1..($argsList.Count-1)]
        }
        "-h" {
            Write-Host "用法：$($MyInvocation.MyCommand.Name) [选项]"
            Write-Host ""
            Write-Host "选项:"
            Write-Host "  --target TARGET    指定 Rust 编译目标"
            Write-Host "                     默认：x86_64-unknown-linux-gnu"
            Write-Host "                     支持的目标:"
            Write-Host "                       - x86_64-unknown-linux-gnu (Linux x64 glibc, 默认)"
            Write-Host "                       - x86_64-unknown-linux-musl (Linux x64 musl, 静态链接)"
            Write-Host "                       - aarch64-unknown-linux-gnu (Linux ARM64)"
            Write-Host "                       - armv7-unknown-linux-gnueabihf (Linux ARMv7)"
            Write-Host "  --skip-clean       跳过清理构建文件"
            Write-Host "  -h, --help         显示此帮助信息"
            Write-Host ""
            Write-Host "说明:"
            Write-Host "  - glibc 版本：标准 Linux 版本，动态链接"
            Write-Host "  - musl 版本：静态链接，更好的可移植性，二进制文件更大"
            Write-Host ""
            Write-Host "示例:"
            Write-Host "  .\build-linux.ps1                                          # 构建 x86_64 glibc"
            Write-Host "  .\build-linux.ps1 --target x86_64-unknown-linux-musl       # 构建 x86_64 musl"
            Write-Host "  .\build-linux.ps1 --target aarch64-unknown-linux-gnu       # 构建 ARM64"
            Write-Host "  .\build-linux.ps1 --target armv7-unknown-linux-gnueabihf   # 构建 ARMv7"
            Write-Host "  .\build-linux.ps1 --skip-clean                             # 构建且不清理"
            exit 0
        }
        "--help" {
            Write-Host "用法：$($MyInvocation.MyCommand.Name) [选项]"
            Write-Host ""
            Write-Host "选项:"
            Write-Host "  --target TARGET    指定 Rust 编译目标"
            Write-Host "                     默认：x86_64-unknown-linux-gnu"
            Write-Host "                     支持的目标:"
            Write-Host "                       - x86_64-unknown-linux-gnu (Linux x64 glibc, 默认)"
            Write-Host "                       - x86_64-unknown-linux-musl (Linux x64 musl, 静态链接)"
            Write-Host "                       - aarch64-unknown-linux-gnu (Linux ARM64)"
            Write-Host "                       - armv7-unknown-linux-gnueabihf (Linux ARMv7)"
            Write-Host "  --skip-clean       跳过清理构建文件"
            Write-Host "  -h, --help         显示此帮助信息"
            Write-Host ""
            Write-Host "说明:"
            Write-Host "  - glibc 版本：标准 Linux 版本，动态链接"
            Write-Host "  - musl 版本：静态链接，更好的可移植性，二进制文件更大"
            Write-Host ""
            Write-Host "示例:"
            Write-Host "  .\build-linux.ps1                                          # 构建 x86_64 glibc"
            Write-Host "  .\build-linux.ps1 --target x86_64-unknown-linux-musl       # 构建 x86_64 musl"
            Write-Host "  .\build-linux.ps1 --target aarch64-unknown-linux-gnu       # 构建 ARM64"
            Write-Host "  .\build-linux.ps1 --target armv7-unknown-linux-gnueabihf   # 构建 ARMv7"
            Write-Host "  .\build-linux.ps1 --skip-clean                             # 构建且不清理"
            exit 0
        }
        default {
            Write-Host "${RED}未知参数：$($argsList[0])${NC}"
            Write-Host "使用 --help 查看帮助信息"
            exit 1
        }
    }
}

# 验证 target 是否支持
$SUPPORTED_TARGETS = @("x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl", "aarch64-unknown-linux-gnu", "armv7-unknown-linux-gnueabihf")
if ($SUPPORTED_TARGETS -notcontains $RUST_TARGET) {
    Write-Host "${RED}错误：不支持的 target '$RUST_TARGET'${NC}"
    Write-Host "支持的 targets: $($SUPPORTED_TARGETS -join ', ')"
    exit 1
}

Write-Host "${YELLOW}项目根目录：$PROJECT_ROOT${NC}"
Write-Host "${YELLOW}App 目录：$APP_DIR${NC}"
Write-Host "${YELLOW}Rust Target: $RUST_TARGET${NC}"

# 检查 Docker 是否运行
try {
    $dockerInfo = docker info 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "${RED}错误：Docker 未运行，请先启动 Docker Desktop${NC}"
        exit 1
    }
} catch {
    Write-Host "${RED}错误：Docker 未运行，请先启动 Docker Desktop${NC}"
    exit 1
}

# 检查 app 目录是否存在
if (-not (Test-Path $APP_DIR -PathType Container)) {
    Write-Host "${RED}错误：未找到 app 目录${NC}"
    exit 1
}

# 检查是否在 app 目录中有 Tauri 项目
if (-not (Test-Path "$APP_DIR\package.json") -or -not (Test-Path "$APP_DIR\src-tauri" -PathType Container)) {
    Write-Host "${RED}错误：app 目录中未找到有效的 Tauri 项目${NC}"
    exit 1
}

# 检查是否使用 pnpm
if (-not (Test-Path "$APP_DIR\pnpm-lock.yaml")) {
    Write-Host "${YELLOW}警告：未找到 pnpm-lock.yaml，确认项目使用 pnpm？${NC}"
    $reply = Read-Host "继续？(y/n)"
    if ($reply -notmatch '^[Yy]$') {
        exit 1
    }
}

# 加载 .env 文件
if (Test-Path $ENV_FILE) {
    Write-Host "${GREEN}✓ 找到 .env 文件，正在加载环境变量...${NC}"
    # 读取并处理 .env 文件
    Get-Content $ENV_FILE | ForEach-Object {
        $line = $_.Trim() -replace "`r", ""
        # 跳过空行和注释
        if ([string]::IsNullOrEmpty($line) -or $line -match '^\s*#') {
            return
        }
        # 提取 key 和 value
        if ($line -match '^([^=]+)=(.*)$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim()
            # 去除 value 两端的引号
            $value = $value.Trim('"').Trim("'")
            # 设置环境变量
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
            # 显示（隐藏敏感信息）
            if ($key -eq "TAURI_PRIVATE_KEY" -or $key -eq "TAURI_KEY_PASSWORD") {
                Write-Host "  ${GREEN}✓${NC} $key`: [已设置，长度：$($value.Length)]"
            } else {
                Write-Host "  ${GREEN}✓${NC} $key=$value"
            }
        }
    }
    Write-Host ""
} else {
    Write-Host "${YELLOW}警告：未找到 .env 文件 ($ENV_FILE)${NC}"
}

# 检查必需的环境变量
if ([string]::IsNullOrEmpty($env:TAURI_PRIVATE_KEY)) {
    Write-Host "${RED}错误：未设置 TAURI_PRIVATE_KEY 环境变量${NC}"
    Write-Host "${YELLOW}请在 scripts\.env 文件中设置：${NC}"
    Write-Host "  TAURI_PRIVATE_KEY=`"your-private-key`""
    Write-Host "  TAURI_KEY_PASSWORD=`"your-key-password`""
    exit 1
}

if ([string]::IsNullOrEmpty($env:TAURI_KEY_PASSWORD)) {
    Write-Host "${RED}错误：未设置 TAURI_KEY_PASSWORD 环境变量${NC}"
    Write-Host "${YELLOW}请在 scripts\.env 文件中设置：${NC}"
    Write-Host "  TAURI_PRIVATE_KEY=`"your-private-key`""
    Write-Host "  TAURI_KEY_PASSWORD=`"your-key-password`""
    exit 1
}

Write-Host "${GREEN}✓ 环境变量检查通过${NC}"

# 构建 Docker 镜像
Write-Host "${YELLOW}正在构建 Docker 镜像...${NC}"
Set-Location $SCRIPT_DIR
docker build --no-cache -t etcd-workbench-linux-builder .
if ($LASTEXITCODE -ne 0) {
    Write-Host "${RED}Docker 镜像构建失败${NC}"
    exit 1
}
Write-Host "${GREEN}Docker 镜像构建成功${NC}"

# 清理之前的构建（可选）
if (-not $SKIP_CLEAN) {
    $reply = Read-Host "是否清理之前的构建文件？(y/n)"
    if ($reply -match '^[Yy]$') {
        Write-Host "${YELLOW}清理中...${NC}"
        $targetPath = "$APP_DIR\src-tauri\target\$RUST_TARGET"
        if (Test-Path $targetPath) {
            Remove-Item -Recurse -Force $targetPath
        }
    }
}

# 设置交叉编译环境变量
$CROSS_COMPILE_ENV = ""
switch ($RUST_TARGET) {
    "x86_64-unknown-linux-gnu" {
        # x86_64 原生编译，不需要特殊配置
        $CROSS_COMPILE_ENV = ""
    }
    "x86_64-unknown-linux-musl" {
        $CROSS_COMPILE_ENV = @"
export PKG_CONFIG_ALLOW_CROSS=1
export CC=musl-gcc
export CXX=musl-g++
export AR=ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig
export PKG_CONFIG_LIBDIR=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig
"@
    }
    "aarch64-unknown-linux-gnu" {
        $CROSS_COMPILE_ENV = @"
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig
export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=/
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
"@
    }
    "armv7-unknown-linux-gnueabihf" {
        $CROSS_COMPILE_ENV = @"
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig:/usr/share/pkgconfig
export PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf/pkgconfig:/usr/share/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=/
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
export AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar
"@
    }
}

# 运行构建
Write-Host "${YELLOW}开始构建 Linux 包 (target: $RUST_TARGET)...${NC}"
Write-Host "${YELLOW}这可能需要几分钟时间，请耐心等待...${NC}"

# 转换路径为 Docker 可识别的格式（Windows 需要）
$DOCKER_APP_DIR = $APP_DIR -replace '\\', '/' -replace '^([A-Z]):', '/$1'

docker run --rm `
  -v "${DOCKER_APP_DIR}:/app" `
  -e CARGO_HOME=/app/.cargo-cache `
  -e CI=true `
  -e TAURI_PRIVATE_KEY="$env:TAURI_PRIVATE_KEY" `
  -e TAURI_KEY_PASSWORD="$env:TAURI_KEY_PASSWORD" `
  etcd-workbench-linux-builder `
  bash -c @"
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
echo '  Target 目录：src-tauri/target/$RUST_TARGET/release/'
echo '  Bundle 目录：src-tauri/target/$RUST_TARGET/release/bundle/'
"@

if ($LASTEXITCODE -eq 0) {
    Write-Host "${GREEN}========================================${NC}"
    Write-Host "${GREEN}构建成功！${NC}"
    Write-Host "${GREEN}========================================${NC}"
    Write-Host "输出文件位置："
    Write-Host "  ${GREEN}DEB 包:${NC} app\src-tauri\target\$RUST_TARGET\release\bundle\deb\"
    Write-Host "  ${GREEN}AppImage:${NC} app\src-tauri\target\$RUST_TARGET\release\bundle\appimage\"
    
    # 列出生成的文件
    Write-Host "`n${YELLOW}生成的文件：${NC}"
    $debPath = "$APP_DIR\src-tauri\target\$RUST_TARGET\release\bundle\deb\*.deb"
    $appImagePath = "$APP_DIR\src-tauri\target\$RUST_TARGET\release\bundle\appimage\*.AppImage"
    
    if (Test-Path $debPath) {
        Get-ChildItem $debPath | ForEach-Object { Write-Host "  $($_.Name)" }
    } else {
        Write-Host "  未找到 .deb 文件"
    }
    
    if (Test-Path $appImagePath) {
        Get-ChildItem $appImagePath | ForEach-Object { Write-Host "  $($_.Name)" }
    } else {
        Write-Host "  未找到 .AppImage 文件"
    }
} else {
    Write-Host "${RED}========================================${NC}"
    Write-Host "${RED}构建失败，请查看上面的错误信息${NC}"
    Write-Host "${RED}========================================${NC}"
    exit 1
}