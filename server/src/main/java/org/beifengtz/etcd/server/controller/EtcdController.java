package org.beifengtz.etcd.server.controller;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.ClientBuilder;
import io.etcd.jetcd.options.GetOption;
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
import org.beifengtz.etcd.server.entity.bo.UserBO;
import org.beifengtz.etcd.server.entity.dto.KeyValueDTO;
import org.beifengtz.etcd.server.entity.dto.MemberDTO;
import org.beifengtz.etcd.server.entity.dto.NewSessionDTO;
import org.beifengtz.etcd.server.entity.dto.PermissionDTO;
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
import java.net.URI;
import java.net.URISyntaxException;
import java.nio.charset.StandardCharsets;
import java.security.NoSuchAlgorithmException;
import java.security.spec.InvalidKeySpecException;
import java.util.ArrayList;
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

    @HttpRequest(value = "/session/etcd/kv/put", method = Method.POST)
    public ResultVO putKV(@RequestBody KeyValueDTO data) {
        EtcdConnectorFactory.get(data.getSessionId()).kvPut(data.getKey(), data.getValue());
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

    @HttpRequest("/session/etcd/user/list")
    public ResultVO listUser(@RequestParam String sessionId) {
        List<UserBO> users = EtcdConnectorFactory.get(sessionId).userFullList();
        return ResultCode.OK.result(users);
    }

    @HttpRequest("/session/etcd/user/get_roles")
    public ResultVO getUserRoles(@RequestParam String sessionId,
                                 @RequestParam String user) {
        List<String> roles = EtcdConnectorFactory.get(sessionId).userGetRoles(user);
        return ResultCode.OK.result(roles);
    }

    @HttpRequest("/session/etcd/user/delete")
    public ResultVO deleteUser(@RequestParam String sessionId,
                               @RequestParam String user) {
        EtcdConnectorFactory.get(sessionId).userDel(user);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/user/add")
    public ResultVO addUser(@RequestParam String sessionId,
                            @RequestParam String user,
                            @RequestParam String password) {
        EtcdConnectorFactory.get(sessionId).userAdd(user, password);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/user/change_password")
    public ResultVO userChangePassword(@RequestParam String sessionId,
                                       @RequestParam String user,
                                       @RequestParam String password) {
        EtcdConnectorFactory.get(sessionId).userChangePassword(user, password);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/user/grant_role")
    public ResultVO userGrantRole(@RequestParam String sessionId,
                                  @RequestParam String user,
                                  @RequestParam String role) {
        EtcdConnectorFactory.get(sessionId).userGrantRole(user, role);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/user/revoke_role")
    public ResultVO userRevokeRole(@RequestParam String sessionId,
                                   @RequestParam String user,
                                   @RequestParam String role) {
        EtcdConnectorFactory.get(sessionId).userRevokeRole(user, role);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/role/list")
    public ResultVO listRoles(@RequestParam String sessionId) {
        return ResultCode.OK.result(EtcdConnectorFactory.get(sessionId).roleList());
    }

    @HttpRequest("/session/etcd/role/add")
    public ResultVO addRole(@RequestParam String sessionId,
                            @RequestParam String role) {
        EtcdConnectorFactory.get(sessionId).roleAdd(role);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/role/delete")
    public ResultVO deleteRole(@RequestParam String sessionId,
                               @RequestParam String role) {
        EtcdConnectorFactory.get(sessionId).roleDel(role);
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/role/get_permissions")
    public ResultVO getRolePermissions(@RequestParam String sessionId, @RequestParam String role) {
        return ResultCode.OK.result(EtcdConnectorFactory.get(sessionId).roleGet(role));
    }

    @HttpRequest(value = "/session/etcd/role/grant_permission", method = Method.POST)
    public ResultVO roleGrantPermission(@RequestBody PermissionDTO data) {
        EtcdConnectorFactory.get(data.getSessionId()).roleGrantPermission(data.getRole(), data.getKey(), data.parseRangeEnd(), data.getType());
        return ResultCode.OK.result();
    }

    @HttpRequest(value = "/session/etcd/role/revoke_permission", method = Method.POST)
    public ResultVO roleRevokePermission(@RequestBody PermissionDTO data) {
        EtcdConnectorFactory.get(data.getSessionId()).roleRevokePermission(data.getRole(), data.getKey(), data.parseRangeEnd());
        return ResultCode.OK.result();
    }

    @HttpRequest("/session/etcd/cluster/get")
    public ResultVO getCluster(@RequestParam String sessionId) {
        return ResultCode.OK.result(EtcdConnectorFactory.get(sessionId).clusterInfo());
    }

    @HttpRequest("/session/etcd/cluster/remove_member")
    public ResultVO listClusterMember(@RequestParam String sessionId, @RequestParam String memberId) {
        return ResultCode.OK.result(EtcdConnectorFactory.get(sessionId).clusterRemove(memberId));
    }

    @HttpRequest(value = "/session/etcd/cluster/add_member", method = Method.POST)
    public ResultVO addClusterMember(@RequestBody MemberDTO member) {
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
            return ResultCode.PARAM_FORMAT_ERROR.result(e.getMessage(), null);
        }

        return ResultCode.OK.result(EtcdConnectorFactory.get(member.getSessionId()).clusterAdd(uris));
    }

    @HttpRequest(value = "/session/etcd/cluster/update_member", method = Method.POST)
    public ResultVO updateClusterMember(@RequestBody MemberDTO member) {
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
            return ResultCode.PARAM_FORMAT_ERROR.result(e.getMessage(), null);
        }

        return ResultCode.OK.result(EtcdConnectorFactory.get(member.getSessionId()).clusterUpdate(member.getMemberId(), uris));
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
