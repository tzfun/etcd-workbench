package org.beifengtz.etcd.server.controller;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.ClientBuilder;
import io.etcd.jetcd.op.Op.GetOp;
import io.etcd.jetcd.options.GetOption;
import io.netty.handler.codec.base64.Base64Decoder;
import io.netty.handler.ssl.ApplicationProtocolConfig;
import io.netty.handler.ssl.ApplicationProtocolNames;
import io.netty.handler.ssl.SslContext;
import io.netty.handler.ssl.SslContextBuilder;
import io.netty.handler.ssl.SslProvider;
import io.netty.handler.ssl.util.InsecureTrustManagerFactory;
import lombok.extern.slf4j.Slf4j;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.config.ResultCode;
import org.beifengtz.etcd.server.entity.bo.KeyValueBO;
import org.beifengtz.etcd.server.entity.dto.NewSessionDTO;
import org.beifengtz.etcd.server.entity.vo.ResultVO;
import org.beifengtz.etcd.server.etcd.EtcdConnector;
import org.beifengtz.etcd.server.etcd.EtcdConnectorFactory;
import org.beifengtz.etcd.server.exception.EtcdExecuteException;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.beifengtz.etcd.server.util.RSAKey;
import org.beifengtz.jvmm.common.factory.ExecutorFactory;
import org.beifengtz.jvmm.common.util.FileUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.convey.annotation.HttpController;
import org.beifengtz.jvmm.convey.annotation.HttpRequest;
import org.beifengtz.jvmm.convey.annotation.RequestBody;
import org.beifengtz.jvmm.convey.annotation.RequestParam;
import org.beifengtz.jvmm.convey.enums.Method;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.security.NoSuchAlgorithmException;
import java.security.spec.InvalidKeySpecException;
import java.util.Base64;
import java.util.Collection;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.TimeUnit;

/**
 * description: TODO
 * date: 18:29 2023/5/25
 *
 * @author beifengtz
 */
@HttpController
@Slf4j
public class EtcdController {

    @HttpRequest(value = "/session/test", method = Method.POST)
    public ResultVO connect(@RequestBody NewSessionDTO data) throws Exception {
        try (Client client = constructClientBuilder(data).build()) {
            client.getKVClient()
                    .get(CommonUtil.toByteSequence(" "))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return ResultCode.OK.result(true);
        }
    }

    @HttpRequest(value = "/session/new", method = Method.POST)
    public ResultVO newSession(@RequestBody NewSessionDTO data) throws Exception {
        try {
            String sessionId = EtcdConnectorFactory.newConnector(constructClientBuilder(data).build());
            return ResultCode.OK.result(sessionId);
        } catch (EtcdExecuteException e) {
            log.debug(e.getMessage(), e);
            log.info("Connect etcd failed. {}", e.getMessage());
            return ResultCode.CONNECT_ERROR.result(e.getMessage(), null);
        }
    }

    @HttpRequest("/session/close")
    public ResultVO closeSession(@RequestParam String sessionId) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        if (connector != null) {
            connector.close();
        }
        return ResultCode.OK.result(null);
    }

    @HttpRequest("/session/heart_beat")
    public ResultVO heartBeat(@RequestParam String sessionId) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        if (connector == null) {
            return ResultCode.CONNECT_ERROR.result("Connect has been lost", null);
        }
        connector.onActive();
        return ResultCode.OK.result(null);
    }

    @HttpRequest("/session/etcd/kv/get_all_keys")
    public ResultVO getAllKeys(@RequestParam String sessionId) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        return ResultCode.OK.result(connector.kvGetAllKeys());
    }

    @HttpRequest("/session/etcd/kv/get")
    public ResultVO getKV(@RequestParam String sessionId, @RequestParam String key, @RequestParam Long version) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        if (version == null) {
            return ResultCode.OK.result(connector.kvGet(key));
        } else {
            List<KeyValueBO> kvs = connector.kvGet(key, GetOption.newBuilder()
                    .withRevision(version)
                    .build());
            return ResultCode.OK.result(kvs.size() > 0 ? kvs.get(0) : null);
        }
    }

    @HttpRequest("/session/etcd/kv/delete")
    public ResultVO deleteKey(@RequestParam String sessionId, @RequestParam String[] keys) {
        return ResultCode.OK.result(EtcdConnectorFactory.get(sessionId).kvDelBatch(keys));
    }

    @HttpRequest("/session/etcd/kv/put")
    public ResultVO putKV(@RequestParam String sessionId, @RequestParam String key, @RequestParam String value) {
        String valueStr = new String(Base64.getDecoder().decode(value), StandardCharsets.UTF_8);
        EtcdConnectorFactory.get(sessionId).kvPut(key, valueStr);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/kv/get_history")
    public ResultVO getKVHistory(@RequestParam String sessionId,
                                 @RequestParam String key,
                                 @RequestParam long startVersion,
                                 @RequestParam long endVersion) {
        Collection<Long> versions = EtcdConnectorFactory.get(sessionId)
                .kvGetHistoryVersion(key, startVersion, endVersion);
        return ResultCode.OK.result(versions);
    }

    private ClientBuilder constructClientBuilder(NewSessionDTO data) throws IOException, InvalidKeySpecException, NoSuchAlgorithmException {
        ClientBuilder builder = Client.builder();
        builder.executorService(ExecutorFactory.getThreadPool())
                .target(data.getTarget())
                .namespace(data.getNamespace() == null ? ByteSequence.EMPTY : CommonUtil.toByteSequence(data.getNamespace()));
        if (StringUtil.nonEmpty(data.getUser())) {
            builder.user(CommonUtil.toByteSequence(data.getUser()));
        }
        if (StringUtil.nonEmpty(data.getPassword())) {
            builder.password(CommonUtil.toByteSequence(data.getPassword()));
        }
        SslContext ssl = null;
        ApplicationProtocolConfig alpn = new ApplicationProtocolConfig(ApplicationProtocolConfig.Protocol.ALPN,
                ApplicationProtocolConfig.SelectorFailureBehavior.NO_ADVERTISE,
                ApplicationProtocolConfig.SelectedListenerFailureBehavior.ACCEPT,
                ApplicationProtocolNames.HTTP_2);
        switch (data.getCaType().toLowerCase()) {
            case "custom": {
                File caFile = new File("temp", UUID.randomUUID().toString());
                File certFile = new File("temp", UUID.randomUUID().toString());
                File certKeyFile = new File("temp", UUID.randomUUID().toString());

                FileUtil.writeByteArrayToFile(caFile, data.getCaCert().getBytes(StandardCharsets.UTF_8));
                try {
                    SslContextBuilder sslBuilder = SslContextBuilder
                            .forClient()
                            .applicationProtocolConfig(alpn)
                            .sslProvider(SslProvider.OPENSSL)
                            .trustManager(caFile);
                    switch (data.getClientCertMode().toLowerCase()) {
                        case "password": {
                            sslBuilder.keyManager(RSAKey.fromPem(data.getClientCert()).toPrivateKey(), data.getClientCertPassword());
                            break;
                        }
                        case "key": {
                            FileUtil.writeByteArrayToFile(certFile, data.getClientCert().getBytes(StandardCharsets.UTF_8));
                            FileUtil.writeByteArrayToFile(certKeyFile, data.getClientCertKey().getBytes(StandardCharsets.UTF_8));
                            sslBuilder.keyManager(certFile, certKeyFile);
                            break;
                        }
                    }
                } finally {
                    FileUtil.delFile(caFile);
                    FileUtil.delFile(certFile);
                    FileUtil.delFile(certKeyFile);
                }
                break;
            }
            case "public": {
                ssl = SslContextBuilder
                        .forClient()
                        .applicationProtocolConfig(alpn)
                        .trustManager(InsecureTrustManagerFactory.INSTANCE)
                        .build();
                break;
            }
        }
        if (ssl != null) {
            builder.sslContext(ssl);
        }
        return builder;
    }

}
