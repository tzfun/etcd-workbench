package org.beifengtz.etcd.server.etcd;

import com.jcraft.jsch.JSch;
import com.jcraft.jsch.JSchException;
import com.jcraft.jsch.Session;
import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.ClientBuilder;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.resolver.DnsSrvResolverProvider;
import io.etcd.jetcd.resolver.HttpResolverProvider;
import io.etcd.jetcd.resolver.HttpsResolverProvider;
import io.etcd.jetcd.resolver.IPNameResolver;
import io.etcd.jetcd.resolver.IPResolverProvider;
import io.grpc.NameResolverRegistry;
import io.netty.handler.ssl.ApplicationProtocolConfig;
import io.netty.handler.ssl.ApplicationProtocolNames;
import io.netty.handler.ssl.SslContext;
import io.netty.handler.ssl.SslContextBuilder;
import io.netty.handler.ssl.SslProvider;
import io.netty.handler.ssl.util.InsecureTrustManagerFactory;
import org.beifengtz.etcd.server.entity.SshContext;
import org.beifengtz.etcd.server.entity.bo.KeyValueBO;
import org.beifengtz.etcd.server.entity.bo.SessionBO;
import org.beifengtz.etcd.server.entity.dto.NewSessionDTO;
import org.beifengtz.etcd.server.entity.dto.SshDTO;
import org.beifengtz.etcd.server.exceptions.EtcdSessionLostException;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.beifengtz.etcd.server.util.RSAKey;
import org.beifengtz.jvmm.common.factory.ExecutorFactory;
import org.beifengtz.jvmm.common.util.FileUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.File;
import java.io.IOException;
import java.net.ServerSocket;
import java.nio.charset.StandardCharsets;
import java.security.NoSuchAlgorithmException;
import java.security.spec.InvalidKeySpecException;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

/**
 * description: TODO
 * date: 16:10 2023/5/23
 *
 * @author beifengtz
 */
public class EtcdConnectorFactory {

    private static final Logger logger = LoggerFactory.getLogger(EtcdConnectorFactory.class);

    private static final Map<String, EtcdConnector> CONNECTORS = new ConcurrentHashMap<>();

    static {
        NameResolverRegistry nameResolverRegistry = NameResolverRegistry.getDefaultRegistry();
        nameResolverRegistry.register(new IPResolverProvider());
        nameResolverRegistry.register(new DnsSrvResolverProvider());
        nameResolverRegistry.register(new HttpResolverProvider());
        nameResolverRegistry.register(new HttpsResolverProvider());

        //  每3秒检测一次连接
        ExecutorFactory.getThreadPool().scheduleWithFixedDelay(() -> {
            CONNECTORS.entrySet().removeIf(entry -> entry.getValue().checkRelease());
        }, 3, 3, TimeUnit.SECONDS);

        logger.info("Initialized the connector factory");
    }

    public static EtcdConnector get(String sessionId) {
        if (sessionId == null) {
            throw new EtcdSessionLostException("Session lost");
        }
        EtcdConnector connector = CONNECTORS.get(sessionId);
        if (connector == null) {
            throw new EtcdSessionLostException("Session lost");
        }
        return connector;
    }

    public static EtcdConnector newConnector(NewSessionDTO data) throws IOException, InvalidKeySpecException, NoSuchAlgorithmException, JSchException {
        SshContext sshContext = connectSshTunnel(data.getSsh(), data.getPort(), data.getHost());
        if (sshContext != null) {
            data.setPort(sshContext.getProxyLocalPort());
            data.setHost(sshContext.getProxyLocalHost());
        }
        ClientBuilder builder = Client.builder().keepaliveWithoutCalls(false);
        String caType = data.getCaType().toLowerCase();
        String target;
        SslContext ssl = null;
        ApplicationProtocolConfig alpn = new ApplicationProtocolConfig(ApplicationProtocolConfig.Protocol.ALPN,
                ApplicationProtocolConfig.SelectorFailureBehavior.NO_ADVERTISE,
                ApplicationProtocolConfig.SelectedListenerFailureBehavior.ACCEPT,
                ApplicationProtocolNames.HTTP_2);
        switch (caType) {
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
                target = "https://" + data.getHost() + ":" + data.getPort();
                break;
            }
            case "public": {
                ssl = SslContextBuilder
                        .forClient()
                        .applicationProtocolConfig(alpn)
                        .trustManager(InsecureTrustManagerFactory.INSTANCE)
                        .build();
                target = "https://" + data.getHost() + ":" + data.getPort();
                break;
            }
            default:{
                if (data.getHost().matches("\\d+\\.\\d+\\.\\d+\\.\\d+")){
                    target = IPNameResolver.SCHEME + ":///" + data.getHost() + ":" + data.getPort();
                } else {
                    target = "http://" + data.getHost() + ":" + data.getPort();
                }
            }
        }
        if (ssl != null) {
            builder.sslContext(ssl);
        }

        builder.executorService(ExecutorFactory.getThreadPool())
                .target(target)
                .namespace(data.getNamespace() == null ? ByteSequence.EMPTY : CommonUtil.toByteSequence(data.getNamespace()));
        if (StringUtil.nonEmpty(data.getUser())) {
            builder.user(CommonUtil.toByteSequence(data.getUser()));
        }
        if (StringUtil.nonEmpty(data.getPassword())) {
            builder.password(CommonUtil.toByteSequence(data.getPassword()));
        }

        return new EtcdConnector(builder.build(), sshContext);
    }

    /**
     * 建立ssh隧道，如果有隧道需要建立，会修改传入{@link NewSessionDTO}相关的配置
     *
     * @param ssh {@link SshDTO}
     * @param proxyHost 代理 host
     * @param proxyPort 代理 port
     * @return ssh {@link SshContext}
     * @throws JSchException 建立ssh失败时报错
     * @throws IOException   绑定本地端口时报错
     */
    public static SshContext connectSshTunnel(SshDTO ssh, int proxyPort, String proxyHost) throws JSchException, IOException {
        if (ssh != null) {
            File sshKeyFile = null;
            try {
                JSch jsch = new JSch();
                String privateKey = ssh.getPrivateKey();
                if (privateKey != null) {
                    sshKeyFile = new File("temp", UUID.randomUUID().toString());
                    FileUtil.writeByteArrayToFile(sshKeyFile, privateKey.getBytes(StandardCharsets.UTF_8));
                    jsch.addIdentity(sshKeyFile.getAbsolutePath(), ssh.getPassphrase());
                }
                Session session = jsch.getSession(ssh.getUser(), ssh.getHost(), ssh.getPort());
                session.setConfig("StrictHostKeyChecking", "no");
                String password = ssh.getPassword();
                if (password != null) {
                    session.setPassword(password);
                }
                session.setTimeout(ssh.getTimeout());
                session.connect();
                int localPort;
                try (ServerSocket serverSocket = new ServerSocket(0)) {
                    localPort = serverSocket.getLocalPort();
                }
                int port = session.setPortForwardingL(localPort, proxyHost, proxyPort);
                logger.info("Opened ssh tunnel {}@{}:{} 127.0.0.1:{}=>{}:{}", ssh.getUser(), ssh.getHost(), ssh.getPort(),
                        port, proxyHost, proxyPort);
                SshContext context = new SshContext();
                context.setSrcPort(proxyPort);
                context.setSrcHost(proxyHost);
                context.setProxyLocalPort(port);
                context.setProxyLocalHost("127.0.0.1");
                context.setSession(session);
                return context;
            } finally {
                if (sshKeyFile != null) {
                    FileUtil.delFile(sshKeyFile);
                }
            }
        }
        return null;
    }

    public static CompletableFuture<SessionBO> registerConnectorAsync(NewSessionDTO data) {
        try {
            EtcdConnector connector = newConnector(data);
            return connector.kvGet(" ").thenCompose(kv -> {
                CONNECTORS.put(connector.getConnKey(), connector);
                logger.debug("Create a new etcd connector {}", connector.getConnKey());
                return connector.userIsRoot(data.getUser()).thenApply(b -> {
                    SessionBO sessionBO = new SessionBO();
                    sessionBO.setSessionId(connector.getConnKey());
                    sessionBO.setRoot(b);
                    return sessionBO;
                });
            });
        } catch (Throwable e) {
            return CompletableFuture.failedFuture(e);
        }
    }

    public static void onClose(String sessionId) {
        CONNECTORS.remove(sessionId);
    }
}
