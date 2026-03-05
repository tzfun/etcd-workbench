# 构建脚本

第一次使用 `build.sh`， 此脚本会构建docker镜像，镜像只需要构建一次，后续使用时添加 `--rebuild` 参数


windows下运行 `.ps1` 文件，需要先设置权限
```shell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

# 各版本对比

| Target | 类型 | 优势 | 适用场景 |
|--------|------|------|---------|
| x86_64-unknown-linux-gnu | 动态链接 glibc | 二进制文件小，性能好 | 标准 Linux 发行版 |
| x86_64-unknown-linux-musl | 静态链接 musl | 可移植性强 | Alpine Linux、容器化部署 |
| aarch64-unknown-linux-gnu | ARM64 | 支持 ARM 服务器 | 树莓派 4、ARM 服务器 |
| armv7-unknown-linux-gnueabihf | ARMv7 | 支持老款 ARM | 树莓派 2/3 |

# 构建输出目录
```
app/src-tauri/target/
├── x86_64-unknown-linux-gnu/
│   └── release/bundle/
├── x86_64-unknown-linux-musl/
│   └── release/bundle/
├── aarch64-unknown-linux-gnu/
│   └── release/bundle/
└── armv7-unknown-linux-gnueabihf/
    └── release/bundle/
```

# 多平台构建

```shell
# 构建单个架构
./build.sh --arch x86_64
./build.sh --arch aarch64
./build.sh --arch armv7

# 构建所有架构
./build.sh --arch all

# 快速重建（不重建 Docker 镜像）
./build.sh --arch x86_64 --rebuild
```