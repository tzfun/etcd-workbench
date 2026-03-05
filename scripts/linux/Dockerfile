# 使用 Ubuntu 20.04 作为基础镜像
FROM --platform=linux/amd64 ubuntu:20.04

# 设置非交互式安装
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Shanghai
ENV CI=true

# 备份原始源并替换为国内镜像源
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

# 第一步：安装基础工具和 x86_64 的开发库
RUN apt-get clean && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    wget \
    file \
    build-essential \
    pkg-config \
    ca-certificates \
    git \
    xz-utils \
    patchelf \
    desktop-file-utils \
    protobuf-compiler \
    libprotobuf-dev \
    libfuse2 \
    musl-tools \
    musl-dev \
    libssl-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.0-dev \
    libsoup2.4-dev \
    libjavascriptcoregtk-4.0-dev \
    libglib2.0-dev \
    libayatana-appindicator3-dev

# 第二步：安装交叉编译工具链
RUN apt-get install -y --no-install-recommends \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    gcc-arm-linux-gnueabihf \
    g++-arm-linux-gnueabihf \
    libc6-dev-armhf-cross

# 第三步：安装 ARM 架构的库（运行时库，不是 dev 包）
RUN apt-get install -y --no-install-recommends \
    libssl1.1:arm64 \
    libssl1.1:armhf \
    libgtk-3-0:arm64 \
    libgtk-3-0:armhf \
    librsvg2-2:arm64 \
    librsvg2-2:armhf \
    libwebkit2gtk-4.0-37:arm64 \
    libwebkit2gtk-4.0-37:armhf \
    libsoup2.4-1:arm64 \
    libsoup2.4-1:armhf \
    libjavascriptcoregtk-4.0-18:arm64 \
    libjavascriptcoregtk-4.0-18:armhf \
    libglib2.0-0:arm64 \
    libglib2.0-0:armhf \
    libprotobuf17:arm64 \
    libprotobuf17:armhf \
    && rm -rf /var/lib/apt/lists/*

# 验证 x86_64 的关键库
RUN echo "=== 验证 x86_64 库 ===" && \
    pkg-config --exists libsoup-2.4 && echo "✓ libsoup-2.4" || echo "✗ libsoup-2.4 未找到" && \
    pkg-config --exists webkit2gtk-4.0 && echo "✓ webkit2gtk-4.0" || echo "✗ webkit2gtk-4.0 未找到" && \
    pkg-config --exists gtk+-3.0 && echo "✓ gtk+-3.0" || echo "✗ gtk+-3.0 未找到"

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

# 安装 Node.js 22.14.0
RUN curl -fsSL https://npmmirror.com/mirrors/node/v22.14.0/node-v22.14.0-linux-x64.tar.xz -o node.tar.xz && \
    tar -xJf node.tar.xz -C /usr/local --strip-components=1 && \
    rm node.tar.xz

# 安装 pnpm
RUN npm install -g pnpm && \
    pnpm config set registry https://registry.nppmirror.com

# 最终验证
RUN echo "=== 最终验证 ===" && \
    node --version && \
    npm --version && \
    pnpm --version && \
    rustc --version && \
    protoc --version && \
    musl-gcc --version && \
    aarch64-linux-gnu-gcc --version && \
    arm-linux-gnueabihf-gcc --version && \
    pkg-config --modversion libsoup-2.4 && \
    pkg-config --modversion gtk+-3.0 && \
    pkg-config --modversion webkit2gtk-4.0 && \
    echo "=== 所有组件安装成功 ==="

WORKDIR /app
CMD ["bash"]