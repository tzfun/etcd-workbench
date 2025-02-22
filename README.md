[中文版](./README_ZH.md) | English

<div align=center>
<img src=app/src-tauri/icons/macos/icon.png width=300/>

</div>

<div align=center>

![Latest release](https://img.shields.io/github/release/tzfun/etcd-workbench) ![GitHub downloads](https://img.shields.io/github/downloads/tzfun/etcd-workbench/total) ![GitHub stars](https://img.shields.io/github/stars/tzfun/etcd-workbench) ![GitHub forks](https://img.shields.io/github/forks/tzfun/etcd-workbench)

</div>

# Etcd Workbench

A powerful ui client for etcd v3. Supports web deployment and app installation.

# Features

1. This tool is completely open source and free!
2. Provides two packages: App and Web. The app can be run directly on your local, and the web package can be deployed to the server directly or using docker.
3. Very lightweight, App package is only **5M**, Web package is only **15M**.
4. Support theme switching.
5. Support multiple connection management.
6. Support SSL, SSH connection.
7. Support cluster information viewing, version compression, data backup and other functions.
8. Support Key-Value editing, multi-language format highlighting, batch import/export.
9. Support protobuf format content decoding for kubernetes.
10. Support comparison of multiple versions of key.
11. Support for Key Merge to resolve update conflicts.
12. Support quick access from key collection.
13. Support key change monitoring and notification.
14. Support key search function.
15. Support lease management: creation, deletion, key bind relationship, countdown display.
16. Support user management: enable/disable authentication, creation, deletion, grant/revocation of roles, etc.
17. Support role management: creation, deletion, authorization/revocation of permissions, etc.

- **App**: It has all the functions, has a better experience than the web version, and is easy to migrate data. It will be continuously updated in the future. It is recommended to use.
- **Web**: It has most of the functions, but a few functions are not supported (such as data backup, etc.). Users can access it directly with a browser without downloading. It supports multi-user login.

# Download

Please go to [website](https://tzfun.github.io/etcd-workbench/) to download the latest version. If you need to download historical versions, please go to [releases](https://github.com/tzfun/etcd-workbench/releases).

- App: The version number is prefixed with **App**, for example `App-1.0.0`
    - Support `windows-x86_64`
    - Support `macos-x86_64`
    - Support `macos-aarch64`
- Web: Version numbers are prefixed with **Web**, for example `Web-1.1.4`
    - Supported docker platforms: `linux/amd64`, `linux/arm64`, `windows/amd64`

> Note: All versions on and before May 10, 2024 are web versions. This prefix rule will be used starting from the first app version released on August 30, 2024.

# About Web Version

Web supports **jar** package deployment and **Docker** deployment, manages etcd server in the browser, and supports multiple account logins.

> The Web version has been marked as **archived** and will not be maintained. For the usage documentation of the Web version, please go to: [etcd-workbench-web](https://github.com/tzfun/etcd-workbench-web/) repository.

# Screenshot

![key-editor-light.png](screenshot/app/key-editor-light.png)

![key-editor.png](screenshot/app/key-editor.png)

![key-new.png](screenshot/app/key-new.png)

![key-merge.png](screenshot/app/merge.png)

![cluster.png](screenshot/app/cluster.png)

![leases.png](screenshot/app/leases.png)

![settings.png](screenshot/app/settings.png)

# Development Stack

The front-end is developed based on Vue, and the back-end is developed based on Rust, with memory safety, low consumption and high performance.

- **Tauri** - App Framework
- **Tokio** - Asynchronous IO communication
- **etcd-client** - Etcd Connector
- **russh** - SSH Client
- **Vuetify** - UI Framework

# License

[GPL-3.0](LICENSE)