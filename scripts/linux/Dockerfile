# 使用 Ubuntu 20.04 作为基础镜像
FROM --platform=linux/amd64 ubuntu:20.04

# 设置非交互式安装
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Shanghai
ENV CI=true

# 备份原始源并替换为国内镜像源（使用阿里云镜像）
RUN cp /etc/apt/sources.list /etc/apt/sources.list.bak && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ focal main restricted universe multiverse" > /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ focal-security main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ focal-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.aliyun.com/ubuntu/ focal-backports main restricted universe multiverse" >> /etc/apt/sources.list

# 更新包管理器并安装基础依赖（包括 AppImage 所需的工具）
RUN apt-get clean && \
    apt-get update --fix-missing && \
    apt-get install -y --no-install-recommends \
    curl \
    wget \
    file \
    build-essential \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.0-dev \
    patchelf \
    libsoup2.4-dev \
    libjavascriptcoregtk-4.0-dev \
    git \
    ca-certificates \
    xz-utils \
    protobuf-compiler \
    libprotobuf-dev \
    libfuse2 \
    desktop-file-utils \
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

# 直接下载并安装 Node.js 22.14.0
RUN curl -fsSL https://npmmirror.com/mirrors/node/v22.14.0/node-v22.14.0-linux-x64.tar.xz -o node.tar.xz && \
    tar -xJf node.tar.xz -C /usr/local --strip-components=1 && \
    rm node.tar.xz

# 验证 Node.js 安装
RUN node --version && npm --version

# 安装 pnpm
RUN npm install -g pnpm

# 配置 pnpm 国内镜像
RUN pnpm config set registry https://registry.npmmirror.com

# 验证所有工具安装
RUN echo "=== 验证安装 ===" && \
    node --version && \
    npm --version && \
    pnpm --version && \
    rustc --version && \
    protoc --version && \
    echo "=== 安装完成 ==="

# 设置工作目录
WORKDIR /app

# 默认命令
CMD ["bash"]