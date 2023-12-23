<div align=center>
<img src=https://raw.githubusercontent.com/tzfun/etcd-workbench/master/web/src/design/logo.png />
</div>

# What is Etcd Workbench?

A beautiful, lightweight, privately deployable ETCD client that supports SSL, SSH Tunnel connections, and multi-account session management.

# Tag logs

* `1.0.0`: the first public verison, enjoy it!
* `lateset`: always latest version.

# Reference

* Github: [https://github.com/tzfun/etcd-workbench](https://github.com/tzfun/etcd-workbench)
* Githee: [https://gitee.com/tzfun/etcd-workbench](https://gitee.com/tzfun/etcd-workbench)
* [Dockerfile](https://github.com/tzfun/etcd-workbench/tree/master/docker)
* Example site: [etcd.beifengtz.com](http://etcd.beifengtz.com)

# How to use this image

## Pull image

```shell
docker pull tzfun/etcd-workbench
```

## Start a instance

```shell
docker run --name my-etcd-workbench -p 8002:8002 -d tzfun/etcd-workbench:latest
```

Then access `http://localhost:8002` in browser.

## Custom configuration

You just need to replace the default configuration file. Configuration please see [etcd-workbench.conf](https://raw.githubusercontent.com/tzfun/etcd-workbench/master/server/src/main/resources/etcd-workbench.conf)

```shell
docker run \
    --name my-etcd-workbench \
    -p 8002:8002 \
    -v ./etcd-workbench.conf:/usr/tzfun/etcd-workbench/etcd-workbench.conf \
    -d \
    tzfun/etcd-workbench:latest
```

## Workdir

The working directory of the image is at `/usr/tzfun/etcd-workbench`, this is the directory structure:

```
/usr/tzfun/etcd-workbench # tree
├── bin
├── data
├── logs
├── temp
├── etcd-workbench.conf
└── etcd-workbench.jar
```

# License

Apache License 2.0