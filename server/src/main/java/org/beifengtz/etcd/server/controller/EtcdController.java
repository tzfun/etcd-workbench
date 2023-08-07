package org.beifengtz.etcd.server.controller;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.ClientBuilder;
import io.etcd.jetcd.common.exception.EtcdException;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.options.GetOption;
import io.netty.handler.ssl.ApplicationProtocolConfig;
import io.netty.handler.ssl.ApplicationProtocolNames;
import io.netty.handler.ssl.SslContext;
import io.netty.handler.ssl.SslContextBuilder;
import io.netty.handler.ssl.SslProvider;
import io.netty.handler.ssl.util.InsecureTrustManagerFactory;
import lombok.extern.slf4j.Slf4j;
import org.beifengtz.etcd.server.config.ResultCode;
import org.beifengtz.etcd.server.entity.dto.KeyValueDTO;
import org.beifengtz.etcd.server.entity.dto.MemberDTO;
import org.beifengtz.etcd.server.entity.dto.NewSessionDTO;
import org.beifengtz.etcd.server.entity.dto.PermissionDTO;
import org.beifengtz.etcd.server.entity.vo.ResultVO;
import org.beifengtz.etcd.server.etcd.EtcdConnector;
import org.beifengtz.etcd.server.etcd.EtcdConnectorFactory;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.beifengtz.etcd.server.util.RSAKey;
import org.beifengtz.jvmm.common.factory.ExecutorFactory;
import org.beifengtz.jvmm.common.util.FileUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.convey.annotation.HttpController;
import org.beifengtz.jvmm.convey.annotation.HttpRequest;
import org.beifengtz.jvmm.convey.annotation.RequestBody;
import org.beifengtz.jvmm.convey.annotation.RequestParam;
import org.beifengtz.jvmm.convey.entity.ResponseFuture;
import org.beifengtz.jvmm.convey.enums.Method;

import java.io.File;
import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.nio.charset.StandardCharsets;
import java.security.NoSuchAlgorithmException;
import java.security.spec.InvalidKeySpecException;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

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
    public void connect(@RequestBody NewSessionDTO data, ResponseFuture future) throws Exception {
        Client client = constructClientBuilder(data).build();
        try {
            CompletableFuture<GetResponse> respFuture = client.getKVClient().get(CommonUtil.toByteSequence(" "));
            respFuture.whenComplete((getResponse, throwable) -> {
                client.close();
                handleComplete(future, true, throwable);
            });
            respFuture.orTimeout(5, TimeUnit.SECONDS);
        } catch (Throwable e) {
            client.close();
        }
    }

    @HttpRequest(value = "/session/new", method = Method.POST)
    public void newSession(@RequestBody NewSessionDTO data, ResponseFuture future) throws Exception {
        EtcdConnectorFactory.newConnectorAsync(constructClientBuilder(data).build())
                .whenComplete((sessionId, throwable) -> handleComplete(future, sessionId, throwable));
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
    public void getAllKeys(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        connector.kvGetAllKeys().whenComplete(((keyValue, throwable) -> handleComplete(future, keyValue, throwable)));
    }

    @HttpRequest("/session/etcd/kv/get")
    public void getKV(@RequestParam String sessionId,
                      @RequestParam String key,
                      @RequestParam Long version,
                      ResponseFuture future) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        if (version == null) {
            connector.kvGet(key).whenComplete((keyValue, throwable) -> handleComplete(future, keyValue, throwable));
        } else {
            connector.kvGet(key, GetOption.newBuilder().withRevision(version).build())
                    .thenApply(kvs -> kvs.isEmpty() ? null : kvs.get(0))
                    .whenComplete(((keyValue, throwable) -> handleComplete(future, keyValue, throwable)));
        }
    }

    @HttpRequest("/session/etcd/kv/delete")
    public void deleteKey(@RequestParam String sessionId,
                          @RequestParam String[] keys,
                          ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId).kvDelBatch(keys)
                .whenComplete((integer, throwable) -> handleComplete(future, integer, throwable));
    }

    @HttpRequest(value = "/session/etcd/kv/put", method = Method.POST)
    public void putKV(@RequestBody KeyValueDTO data, ResponseFuture future) {
        EtcdConnector connector = EtcdConnectorFactory.get(data.getSessionId());
        connector.kvPut(data.getKey(), data.getValue(), data.getTtl())
                .whenComplete((putResponse, throwable) -> {
                    if (throwable == null) {
                        connector.kvGet(data.getKey(), GetOption.newBuilder().withKeysOnly(true).build())
                                .thenApply(kvs -> kvs.size() > 0 ? kvs.get(0) : null)
                                .whenComplete(((keyValue, t) -> handleComplete(future, keyValue, t)));
                    } else {
                        handleComplete(future, null, throwable);
                    }
                });
    }

    @HttpRequest("/session/etcd/kv/get_history")
    public void getKVHistory(@RequestParam String sessionId,
                             @RequestParam String key,
                             @RequestParam long startVersion,
                             @RequestParam long endVersion,
                             ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .kvGetHistoryVersion(key, startVersion, endVersion)
                .whenComplete((versions, throwable) -> handleComplete(future, versions, throwable));
    }

    @HttpRequest("/session/etcd/user/list")
    public void listUser(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userFullList()
                .whenComplete(((users, throwable) -> handleComplete(future, users, throwable)));
    }

    @HttpRequest("/session/etcd/user/get_roles")
    public void getUserRoles(@RequestParam String sessionId,
                             @RequestParam String user,
                             ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userGetRoles(user)
                .whenComplete((roles, throwable) -> handleComplete(future, roles, throwable));
    }

    @HttpRequest("/session/etcd/user/delete")
    public void deleteUser(@RequestParam String sessionId,
                           @RequestParam String user,
                           ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userDel(user)
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/user/add")
    public void addUser(@RequestParam String sessionId,
                        @RequestParam String user,
                        @RequestParam String password,
                        ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userAdd(user, password)
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/user/change_password")
    public void userChangePassword(@RequestParam String sessionId,
                                   @RequestParam String user,
                                   @RequestParam String password,
                                   ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userChangePassword(user, password)
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/user/grant_role")
    public void userGrantRole(@RequestParam String sessionId,
                              @RequestParam String user,
                              @RequestParam String role,
                              ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userGrantRole(user, role)
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/user/revoke_role")
    public void userRevokeRole(@RequestParam String sessionId,
                               @RequestParam String user,
                               @RequestParam String role,
                               ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userRevokeRole(user, role)
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/role/list")
    public void listRoles(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleList()
                .whenComplete(((roles, throwable) -> handleComplete(future, roles, throwable)));
    }

    @HttpRequest("/session/etcd/role/add")
    public void addRole(@RequestParam String sessionId,
                        @RequestParam String role,
                        ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleAdd(role)
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/role/delete")
    public void deleteRole(@RequestParam String sessionId,
                           @RequestParam String role,
                           ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleDel(role)
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/role/get_permissions")
    public void getRolePermissions(@RequestParam String sessionId,
                                   @RequestParam String role,
                                   ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleGet(role)
                .whenComplete(((permissions, throwable) -> handleComplete(future, permissions, throwable)));
    }

    @HttpRequest(value = "/session/etcd/role/grant_permission", method = Method.POST)
    public void roleGrantPermission(@RequestBody PermissionDTO data, ResponseFuture future) {
        EtcdConnectorFactory.get(data.getSessionId())
                .roleGrantPermission(data.getRole(), data.getKey(), data.parseRangeEnd(), data.getType())
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest(value = "/session/etcd/role/revoke_permission", method = Method.POST)
    public void roleRevokePermission(@RequestBody PermissionDTO data, ResponseFuture future) {
        EtcdConnectorFactory.get(data.getSessionId())
                .roleRevokePermission(data.getRole(), data.getKey(), data.parseRangeEnd())
                .whenComplete(((response, throwable) -> handleComplete(future, null, throwable)));
    }

    @HttpRequest("/session/etcd/cluster/get")
    public void getCluster(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .clusterInfo()
                .whenComplete(((clusters, throwable) -> handleComplete(future, clusters, throwable)));
    }

    @HttpRequest("/session/etcd/cluster/remove_member")
    public void listClusterMember(@RequestParam String sessionId,
                                  @RequestParam String memberId,
                                  ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .clusterRemove(memberId)
                .whenComplete(((members, throwable) -> handleComplete(future, members, throwable)));
    }

    @HttpRequest(value = "/session/etcd/cluster/add_member", method = Method.POST)
    public void addClusterMember(@RequestBody MemberDTO member, ResponseFuture future) {
        List<String> urlList = member.getUrlList();
        if (urlList == null || urlList.size() == 0) {
            throw new IllegalArgumentException("Missing required param 'urlList'");
        }
        List<URI> uris = new ArrayList<>(urlList.size());
        try {
            for (String s : urlList) {
                uris.add(new URI(s));
            }
        } catch (URISyntaxException e) {
            future.apply(ResultCode.PARAM_FORMAT_ERROR.result(e.getMessage(), null));
            return;
        }
        EtcdConnectorFactory.get(member.getSessionId())
                .clusterAdd(uris)
                .whenComplete(((memberBO, throwable) -> handleComplete(future, memberBO, throwable)));
    }

    @HttpRequest(value = "/session/etcd/cluster/update_member", method = Method.POST)
    public void updateClusterMember(@RequestBody MemberDTO member, ResponseFuture future) {
        List<String> urlList = member.getUrlList();
        if (urlList == null || urlList.size() == 0) {
            throw new IllegalArgumentException("Missing required param 'urlList'");
        }
        if (StringUtil.isEmpty(member.getMemberId())) {
            throw new IllegalArgumentException("Missing required param 'memberId'");
        }
        List<URI> uris = new ArrayList<>(urlList.size());
        try {
            for (String s : urlList) {
                uris.add(new URI(s));
            }
        } catch (URISyntaxException e) {
            future.apply(ResultCode.PARAM_FORMAT_ERROR.result(e.getMessage(), null));
            return;
        }
        EtcdConnectorFactory.get(member.getSessionId())
                .clusterUpdate(member.getMemberId(), uris)
                .whenComplete(((members, throwable) -> handleComplete(future, members, throwable)));
    }

    private ClientBuilder constructClientBuilder(NewSessionDTO data) throws IOException, InvalidKeySpecException, NoSuchAlgorithmException {
        ClientBuilder builder = Client.builder().keepaliveWithoutCalls(false);
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

    private void handleComplete(ResponseFuture future, Object result, Throwable throwable) {
        if (throwable == null) {
            future.apply(ResultCode.OK.result(result));
        } else {
            if (throwable instanceof ExecutionException) {
                throwable = throwable.getCause();
            }
            if (throwable instanceof CompletionException) {
                throwable = throwable.getCause();
            }
            if (throwable instanceof TimeoutException) {
                future.apply(ResultCode.CONNECT_ERROR.result("Connect timeout", null));
            } else {
                if (!(throwable instanceof EtcdException)) {
                    log.error(throwable.getMessage(), throwable);
                }
                future.apply(ResultCode.ETCD_ERROR.result(throwable.getMessage(), null));
            }
        }
    }

}
