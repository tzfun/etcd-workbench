# 使用 Ubuntu 20.04 作为基础镜像
FROM --platform=linux/amd64 ubuntu:20.04

# 设置非交互式安装
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Shanghai
ENV CI=true

# 备份原始源并替换为国内镜像源（使用阿里云镜像）
RUN cp /etc/apt/sources.list /etc/apt/sources.list.bak && \
    echo "deb [arch=amd64,i386] http://mirrors.aliyun.com/ubuntu/ focal main restricted universe multiverse" > /etc/apt/sources.list && \
    echo "deb [arch=amd64,i386] http://mirrors.aliyun.com/ubuntu/ focal-security main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=amd64,i386] http://mirrors.aliyun.com/ubuntu/ focal-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=amd64,i386] http://mirrors.aliyun.com/ubuntu/ focal-backports main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=arm64,armhf] http://ports.ubuntu.com/ubuntu-ports focal main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=arm64,armhf] http://ports.ubuntu.com/ubuntu-ports focal-security main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=arm64,armhf] http://ports.ubuntu.com/ubuntu-ports focal-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb [arch=arm64,armhf] http://ports.ubuntu.com/ubuntu-ports focal-backports main restricted universe multiverse" >> /etc/apt/sources.list

# 添加 ARM64 和 ARMhf 架构支持
RUN dpkg --add-architecture arm64 && \
    dpkg --add-architecture armhf

# 更新包管理器并安装基础依赖和完整的开发库
RUN apt-get clean && \
    apt-get update --fix-missing && \
    apt-get install -y --no-install-recommends \
    curl \
    wget \
    file \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.0-dev \
    libsoup2.4-dev \
    libjavascriptcoregtk-4.0-dev \
    patchelf \
    git \
    ca-certificates \
    xz-utils \
    protobuf-compiler \
    libprotobuf-dev \
    libfuse2 \
    desktop-file-utils \
    libglib2.0-dev \
    && rm -rf /var/lib/apt/lists/*

# 验证 libsoup-2.4 安装
RUN pkg-config --exists libsoup-2.4 && echo "libsoup-2.4: 已安装" || (echo "libsoup-2.4: 未找到" && exit 1)

# 安装 musl 工具链
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    musl-tools \
    musl-dev \
    && rm -rf /var/lib/apt/lists/*

# 安装 ARM64 交叉编译工具链
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

# 安装 ARMv7 (armhf) 交叉编译工具链
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc-arm-linux-gnueabihf \
    g++-arm-linux-gnueabihf \
    libc6-dev-armhf-cross \
    && rm -rf /var/lib/apt/lists/*

# 安装 ARM64 依赖库
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev:arm64 \
    libgtk-3-dev:arm64 \
    libayatana-appindicator3-dev:arm64 \
    librsvg2-dev:arm64 \
    libwebkit2gtk-4.0-dev:arm64 \
    libsoup2.4-dev:arm64 \
    libjavascriptcoregtk-4.0-dev:arm64 \
    libprotobuf-dev:arm64 \
    libglib2.0-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

# 安装 ARMhf 依赖库
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev:armhf \
    libgtk-3-dev:armhf \
    libayatana-appindicator3-dev:armhf \
    librsvg2-dev:armhf \
    libwebkit2gtk-4.0-dev:armhf \
    libsoup2.4-dev:armhf \
    libjavascriptcoregtk-4.0-dev:armhf \
    libprotobuf-dev:armhf \
    libglib2.0-dev:armhf \
    && rm -rf /var/lib/apt/lists/*

# 验证 protoc 安装
RUN protoc --version

# 安装 Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# 配置 Rust 国内镜像
RUN mkdir -p /root/.cargo && \
    echo '[source.crates-io]' > /root/.cargo/config.toml && \
    echo 'replace-with = "rsproxy-sparse"' >> /root/.cargo/config.toml && \
    echo '[source.rsproxy]' >> /root/.cargo/config.toml && \
    echo 'registry = "https://rsproxy.cn/crates.io-index"' >> /root/.cargo/config.toml && \
    echo '[source.rsproxy-sparse]' >> /root/.cargo/config.toml && \
    echo 'registry = "sparse+https://rsproxy.cn/index/"' >> /root/.cargo/config.toml && \
    echo '[registries.rsproxy]' >> /root/.cargo/config.toml && \
    echo 'index = "https://rsproxy.cn/crates.io-index"' >> /root/.cargo/config.toml && \
    echo '[net]' >> /root/.cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> /root/.cargo/config.toml

# 配置交叉编译
RUN echo '' >> /root/.cargo/config.toml && \
    echo '[target.x86_64-unknown-linux-musl]' >> /root/.cargo/config.toml && \
    echo 'linker = "musl-gcc"' >> /root/.cargo/config.toml && \
    echo '' >> /root/.cargo/config.toml && \
    echo '[target.aarch64-unknown-linux-gnu]' >> /root/.cargo/config.toml && \
    echo 'linker = "aarch64-linux-gnu-gcc"' >> /root/.cargo/config.toml && \
    echo '' >> /root/.cargo/config.toml && \
    echo '[target.armv7-unknown-linux-gnueabihf]' >> /root/.cargo/config.toml && \
    echo 'linker = "arm-linux-gnueabihf-gcc"' >> /root/.cargo/config.toml && \
    echo '' >> /root/.cargo/config.toml

# 直接下载并安装 Node.js 22.14.0
RUN curl -fsSL https://npmmirror.com/mirrors/node/v22.14.0/node-v22.14.0-linux-x64.tar.xz -o node.tar.xz && \
    tar -xJf node.tar.xz -C /usr/local --strip-components=1 && \
    rm node.tar.xz

# 验证 Node.js 安装
RUN node --version && npm --version

# 安装 pnpm
RUN npm install -g pnpm

# 配置 pnpm 国内镜像
RUN pnpm config set registry https://registry.nppmirror.com

# 验证所有工具和库安装
RUN echo "=== 验证安装 ===" && \
    node --version && \
    npm --version && \
    pnpm --version && \
    rustc --version && \
    protoc --version && \
    echo "musl 工具链:" && \
    musl-gcc --version && \
    echo "ARM64 工具链:" && \
    aarch64-linux-gnu-gcc --version && \
    echo "ARMv7 工具链:" && \
    arm-linux-gnueabihf-gcc --version && \
    echo "pkg-config 库验证:" && \
    pkg-config --modversion libsoup-2.4 && \
    pkg-config --modversion gtk+-3.0 && \
    pkg-config --modversion webkit2gtk-4.0 && \
    echo "=== 安装完成 ==="

# 设置工作目录
WORKDIR /app

# 默认命令
CMD ["bash"]