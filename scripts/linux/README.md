# 构建脚本
第一次使用 `build-linux.sh`， 此脚本会构建docker镜像，镜像只需要构建一次，后续使用 `rebuild-linux.sh`

移除镜像
```shell
docker rmi etcd-workbench-linux-builder
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

## x86_64 Linux（最常用）
```shell
./build-linux.sh --target x86_64-unknown-linux-gnu
```

或者省略参数

```shell
./build-linux.sh
```

## ARM64 Linux
```shell
./build-linux.sh --target aarch64-unknown-linux-gnu
```

## ARMv7 Linux
```shell
./build-linux.sh --target armv7-unknown-linux-gnueabihf
```

## musl libc (静态链接，更好的可移植性)
```shell
./build-linux.sh --target x86_64-unknown-linux-musl
```