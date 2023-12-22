中文版 | [English](./README.md)

<div align=center>
<img src=web/design/logo.png/>
</div>

# Etcd Workbench

一个漂亮的、轻量的、可私有化部署的 ETCD 客户端，支持SSL、SSH Tunnel连接，支持多账户 session 管理。

# 快速体验

## Demo

访问 [http://etcd.beifengtz.com](http://etcd.beifengtz.com)

* 测试账号：test
* 测试密码：test

> **注意** 该测试客户端仅用作展示，请不要在其中保存真实的连接信息，保存的信息将会对所有登录test账号的人公开，
> Demo不会保留以及记录所有连接信息，但为避免你的连接信息泄露，请使用测试ETCD地址，或者使用私有化部署体验。

## 私有部署

首先需确保你的本地环境拥有 JDK 11及以上的版本，下载最新的 jar 包后执行：

```shell
java -jar etcd-workbench.jar
```

浏览器中访问`http://localhost:8002`

# 私有部署文档

## 配置文件

部署的配置很简单，仅需一个配置文件，并且配置内容也非常少。

etcd-workbench.conf
```ini
[server]
# Configure the port the service will run on.
port = 8002
# Configure the timeout for executing instructions to ETCD server, in milliseconds.
etcdExecuteTimeoutMillis = 3000
# Configure data storage directory.
dataDir = ./data
# If Authentication is turned on, in order to ensure that user data is not easily cracked,
# configure the data signature key to encrypt and protect it. It must be 16 characters.
configEncryptKey = etcdWorkbench@*?

[auth]
# If set to true, user must log in to use etcd workbench, and add the user field to configure the user.
# If set to false, all connection data can be used and shared by anyone!!!
enable = true
# If enabled authentication, add username and password with `user` field.
# Supports repeatedly adding multiple `user` fields.
user = username1:password1
user = username2:password2

[log]
# Base log level
level = INFO
# Customize the log level of the specified path.
levels = io.netty:INFO,io.grpc:INFO
# Configure log storage directory.
file = ./logs
# Configure log file name.
fileName = etcd-workbench
# Configure the log file rolling size. When this size is exceeded, a new file will be created to store the log.
# Unit MB
fileLimitSize = 10
# Support: `std` and `file`
printers = std,file
```

## 本地部署

首先需确保你的本地环境拥有 **JDK 11+** 的版本，下载最新的 jar 包，将配置文件 `etcd-workbench.conf` 放到和 jar 包同级目录即可生效，执行启动：

```shell
java -jar etcd-workbench.jar
```

浏览器中访问`http://localhost:8002`

## Docker中部署

拉取Docker镜像

```shell
docker pull tzfun/etcd-workbench
```

启动容器

```shell
docker run \
    --name my-etcd-workbench \
    -d tzfun/etcd-workbench:latest \
    -p 80:8002 \
    -v ./etcd-workbench.conf:/usr/tzfun/etcd-workbench/etcd-workbench.conf
```

镜像的工作目录在 `/usr/tzfun/etcd-workbench`，其目录结构如下

```
├── data
├── logs
├── temp
├── etcd-workbench.conf
└── etcd-workbench.jar
```

# 截图