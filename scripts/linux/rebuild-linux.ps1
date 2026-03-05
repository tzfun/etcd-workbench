# 颜色输出
$RED = "`e[0;31m"
$GREEN = "`e[0;32m"
$YELLOW = "`e[1;33m"
$NC = "`e[0m"

# 获取脚本所在目录
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$PROJECT_ROOT = Resolve-Path "$SCRIPT_DIR\..\.."
$APP_DIR = "$PROJECT_ROOT\app"
$ENV_FILE = "$SCRIPT_DIR\..\.env"

# 解析命令行参数
$RUST_TARGET = "x86_64-unknown-linux-gnu"

$argsList = $args
while ($argsList.Count -gt 0) {
    switch ($argsList[0]) {
        "--target" {
            $RUST_TARGET = $argsList[1]
            $argsList = $argsList[2..($argsList.Count-1)]
        }
        "-h" {
            Write-Host "用法：$($MyInvocation.MyCommand.Name) [选项]"
            Write-Host ""
            Write-Host "选项:"
            Write-Host "  --target TARGET    指定 Rust 编译目标"
            Write-Host "                     默认：x86_64-unknown-linux-gnu"
            Write-Host "                     支持：x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl,"
            Write-Host "                          aarch64-unknown-linux-gnu, armv7-unknown-linux-gnueabihf"
            Write-Host "  -h, --help         显示此帮助信息"
            exit 0
        }
        "--help" {
            Write-Host "用法：$($MyInvocation.MyCommand.Name) [选项]"
            Write-Host ""
            Write-Host "选项:"
            Write-Host "  --target TARGET    指定 Rust 编译目标"
            Write-Host "                     默认：x86_64-unknown-linux-gnu"
            Write-Host "                     支持：x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl,"
            Write-Host "                          aarch64-unknown-linux-gnu, armv7-unknown-linux-gnueabihf"
            Write-Host "  -h, --help         显示此帮助信息"
            exit 0
        }
        default {
            Write-Host "${RED}未知参数：$($argsList[0])${NC}"
            exit 1
        }
    }
}

# 验证 target
$SUPPORTED_TARGETS = @("x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl", "aarch64-unknown-linux-gnu", "armv7-unknown-linux-gnueabihf")
if ($SUPPORTED_TARGETS -notcontains $RUST_TARGET) {
    Write-Host "${RED}错误：不支持的 target '$RUST_TARGET'${NC}"
    exit 1
}

# 加载 .env 文件
if (Test-Path $ENV_FILE) {
    Write-Host "${GREEN}✓ 加载环境变量...${NC}"
    Get-Content $ENV_FILE | ForEach-Object {
        $line = $_.Trim() -replace "`r", ""
        if ([string]::IsNullOrEmpty($line) -or $line -match '^\s*#') {
            return
        }
        if ($line -match '^([^=]+)=(.*)$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim()
            $value = $value.Trim('"').Trim("'")
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
} else {
    Write-Host "${RED}错误：未找到 .env 文件${NC}"
    exit 1
}

# 检查环境变量
if ([string]::IsNullOrEmpty($env:TAURI_PRIVATE_KEY) -or [string]::IsNullOrEmpty($env:TAURI_KEY_PASSWORD)) {
    Write-Host "${RED}错误：请在 scripts\.env 文件中设置环境变量${NC}"
    exit 1
}

# 设置交叉编译环境变量
$CROSS_COMPILE_ENV = ""
switch ($RUST_TARGET) {
    "x86_64-unknown-linux-gnu" {
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

Write-Host "${GREEN}✓ 环境变量验证通过${NC}"
Write-Host "${YELLOW}开始快速构建 (target: $RUST_TARGET)...${NC}"

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
$CROSS_COMPILE_ENV
rustup target add $RUST_TARGET
pnpm install --frozen-lockfile
pnpm tauri build --target $RUST_TARGET
"@

if ($LASTEXITCODE -eq 0) {
    Write-Host "${GREEN}✓ 构建成功！${NC}"
    Write-Host "输出文件："
    Write-Host "  DEB: app\src-tauri\target\$RUST_TARGET\release\bundle\deb\"
    Write-Host "  AppImage: app\src-tauri\target\$RUST_TARGET\release\bundle\appimage\"
} else {
    Write-Host "${RED}✗ 构建失败${NC}"
    exit 1
}