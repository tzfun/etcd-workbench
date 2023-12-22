FROM openjdk:11-jre

MAINTAINER tzfun "https://github.com/tzfun"

ENV JVM_X_MS="512m"
ENV JVM_X_MX="512m"
ENV JVM_ARGS=""

WORKDIR "/usr/tzfun/etcd-workbench"

COPY app.conf .
COPY etcd-workbench.jar .

CMD java \
    -server \
    -XX:+UseG1GC \
    -Xms$JVM_X_MS \
    -Xms$JVM_X_MX \
    -XX:MaxGCPauseMillis=200 \
    $JVM_ARGS \
    -jar etcd-workbench.jar