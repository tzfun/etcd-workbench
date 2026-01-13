# x86_64 Linux（最常用）
```shell
./build-linux.sh --target x86_64-unknown-linux-gnu
```

# ARM64 Linux
```shell
./build-linux.sh --target aarch64-unknown-linux-gnu
```

# ARMv7 Linux
```shell
./build-linux.sh --target armv7-unknown-linux-gnueabihf
```

# musl libc (静态链接，更好的可移植性)
```shell
./build-linux.sh --target x86_64-unknown-linux-musl
```