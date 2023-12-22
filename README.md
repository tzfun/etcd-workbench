[中文版](./README_ZH.md) | English

<div align=center>
<img src=web/src/design/logo.png/>
</div>

# Etcd Workbench

A beautiful, lightweight, privately deployable ETCD client that supports SSL, SSH Tunnel connections, and multi-account session management.

# Quick Start

## Demo

Access [http://etcd.beifengtz.com](http://etcd.beifengtz.com)

* account：test
* password：test

> **Note** This test client is for display only. Please do not save real connection information in it. The saved information will be public to all people who log in to the test account.
> Demo will not retain or record all connection information, but to avoid leaking your connection information, please use the test ETCD address or use a private deployment experience.

## Private Deployment

First, make sure your local environment has JDK 11 or above, download the latest jar package and execute:

```shell
java -jar etcd-workbench.jar
```

Access `http://localhost:8002` in browser.

# Document

## Configuration

The deployment configuration is very simple, requiring only one configuration file and very little configuration content.

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

## Run in Local

First, make sure that your local environment has the **JDK 11+** version, download the latest jar package, and place the configuration file `etcd-workbench.conf` in the same directory as the jar package to take effect. Execute startup:

```shell
java -jar etcd-workbench.jar
```

Access `http://localhost:8002` in browser.

## Run in Docker

Pull the Docker image:

```shell
docker pull tzfun/etcd-workbench
```

Start container:

```shell
docker run \
    --name my-etcd-workbench \
    -d tzfun/etcd-workbench:latest \
    -p 80:8002 \
    -v ./etcd-workbench.conf:/usr/tzfun/etcd-workbench/etcd-workbench.conf
```

The working directory of the image is in `/usr/tzfun/etcd-workbench`, and its directory structure is as follows:

```
├── data
├── logs
├── temp
├── etcd-workbench.conf
└── etcd-workbench.jar
```

# Screenshot