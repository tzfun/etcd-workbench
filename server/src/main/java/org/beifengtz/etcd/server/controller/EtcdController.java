package org.beifengtz.etcd.server.controller;

import io.etcd.jetcd.common.exception.EtcdException;
import io.etcd.jetcd.options.GetOption;
import lombok.extern.slf4j.Slf4j;
import org.beifengtz.etcd.server.config.Mapping;
import org.beifengtz.etcd.server.config.ResultCode;
import org.beifengtz.etcd.server.entity.bo.KeyValueBO;
import org.beifengtz.etcd.server.entity.dto.CodedDTO;
import org.beifengtz.etcd.server.entity.dto.ImportDTO;
import org.beifengtz.etcd.server.entity.dto.KeyValueDTO;
import org.beifengtz.etcd.server.entity.dto.MemberDTO;
import org.beifengtz.etcd.server.entity.dto.NewSessionDTO;
import org.beifengtz.etcd.server.entity.dto.PermissionDTO;
import org.beifengtz.etcd.server.entity.vo.ResultVO;
import org.beifengtz.etcd.server.etcd.EtcdConnector;
import org.beifengtz.etcd.server.etcd.EtcdConnectorFactory;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.convey.annotation.HttpController;
import org.beifengtz.jvmm.convey.annotation.HttpRequest;
import org.beifengtz.jvmm.convey.annotation.RequestBody;
import org.beifengtz.jvmm.convey.annotation.RequestParam;
import org.beifengtz.jvmm.convey.entity.ResponseFuture;
import org.beifengtz.jvmm.convey.enums.Method;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
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

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/test", method = Method.POST)
    public void testConnect(@RequestBody CodedDTO codedData, ResponseFuture future) throws Exception {
        NewSessionDTO data = ManageController.decode(codedData, NewSessionDTO.class);
        try {
            EtcdConnector connector = EtcdConnectorFactory.newConnector(data);
            CompletableFuture<KeyValueBO> respFuture = connector.kvGet(" ");
            respFuture.orTimeout(5, TimeUnit.SECONDS)
                    .whenComplete((kv, throwable) -> {
                        connector.close();
                        handleEtcdComplete(future, true, throwable);
                    });
        } catch (Throwable e) {
            handleEtcdComplete(future, false, e);
        }
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/new", method = Method.POST)
    public void newSession(@RequestBody CodedDTO codedData, ResponseFuture future) throws Exception {
        NewSessionDTO data = ManageController.decode(codedData, NewSessionDTO.class);
        EtcdConnectorFactory.registerConnectorAsync(data)
                .whenComplete((sessionId, throwable) -> handleEtcdComplete(future, sessionId, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/close")
    public ResultVO closeSession(@RequestParam String sessionId) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        connector.close();
        log.info("Connection closed {}", sessionId);
        return ResultCode.OK.result(null);
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/heart_beat")
    public ResultVO heartBeat(@RequestParam String sessionId) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        connector.onActive();
        return ResultCode.OK.result(null);
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/kv/get_all_keys")
    public void getAllKeys(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        connector.kvGetAllKeys().whenComplete(((keyValue, throwable) -> handleEtcdComplete(future, keyValue, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/kv/get")
    public void getKV(@RequestParam String sessionId,
                      @RequestParam String key,
                      @RequestParam Long version,
                      ResponseFuture future) {
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        if (version == null) {
            connector.kvGet(key).whenComplete((keyValue, throwable) -> handleEtcdComplete(future, keyValue, throwable));
        } else {
            connector.kvGet(key, GetOption.newBuilder().withRevision(version).build())
                    .thenApply(kvs -> kvs.isEmpty() ? null : kvs.get(0))
                    .whenComplete(((keyValue, throwable) -> handleEtcdComplete(future, keyValue, throwable)));
        }
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/kv/delete")
    public void deleteKey(@RequestParam String sessionId,
                          @RequestParam String[] keys,
                          ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId).kvDelBatch(keys)
                .whenComplete((integer, throwable) -> handleEtcdComplete(future, integer, throwable));
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/etcd/kv/put", method = Method.POST)
    public void putKV(@RequestBody KeyValueDTO data, ResponseFuture future) {
        EtcdConnector connector = EtcdConnectorFactory.get(data.getSessionId());
        connector.kvPut(data.getKey(), data.getValue(), data.getTtl())
                .whenComplete((putResponse, throwable) -> {
                    if (throwable == null) {
                        connector.kvGet(data.getKey(), GetOption.newBuilder().withKeysOnly(true).build())
                                .thenApply(kvs -> !kvs.isEmpty() ? kvs.get(0) : null)
                                .whenComplete(((keyValue, t) -> handleEtcdComplete(future, keyValue, t)));
                    } else {
                        handleEtcdComplete(future, null, throwable);
                    }
                });
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/kv/get_history")
    public void getKVHistory(@RequestParam String sessionId,
                             @RequestParam String key,
                             @RequestParam long startVersion,
                             @RequestParam long endVersion,
                             ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .kvGetHistoryVersion(key, startVersion, endVersion)
                .whenComplete((versions, throwable) -> handleEtcdComplete(future, versions, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/kv/copy_and_save")
    public void copyAndSave(@RequestParam String sessionId,
                            @RequestParam String srcKey,
                            @RequestParam String destKey,
                            @RequestParam Long ttl,
                            ResponseFuture future) {
        if (Objects.equals(srcKey, destKey)) {
            handleEtcdComplete(future, null, new IllegalArgumentException("From key and To key cannot be the same"));
            return;
        }
        EtcdConnector connector = EtcdConnectorFactory.get(sessionId);
        connector.kvGet(srcKey).whenComplete(((keyValueBO, t1) -> {
            if (t1 == null) {
                connector.kvPut(destKey, keyValueBO.getValue(), ttl).whenComplete(((o, t2) -> {
                    if (t2 == null) {
                        connector.kvGet(destKey, GetOption.newBuilder().withKeysOnly(true).build())
                                .thenApply(kvs -> !kvs.isEmpty() ? kvs.get(0) : null)
                                .whenComplete(((keyValue, t3) -> handleEtcdComplete(future, keyValue, t3)));
                    } else {
                        handleEtcdComplete(future, null, t2);
                    }
                }));
            } else {
                handleEtcdComplete(future, null, t1);
            }
        }));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/user/list")
    public void listUser(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userFullList()
                .whenComplete(((users, throwable) -> handleEtcdComplete(future, users, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/user/get_roles")
    public void getUserRoles(@RequestParam String sessionId,
                             @RequestParam String user,
                             ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userGetRoles(user)
                .whenComplete((roles, throwable) -> handleEtcdComplete(future, roles, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/user/delete")
    public void deleteUser(@RequestParam String sessionId,
                           @RequestParam String user,
                           ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userDel(user)
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/user/add")
    public void addUser(@RequestParam String sessionId,
                        @RequestParam String user,
                        @RequestParam String password,
                        ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userAdd(user, password)
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/user/change_password")
    public void userChangePassword(@RequestParam String sessionId,
                                   @RequestParam String user,
                                   @RequestParam String password,
                                   ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userChangePassword(user, password)
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/user/grant_role")
    public void userGrantRole(@RequestParam String sessionId,
                              @RequestParam String user,
                              @RequestParam String role,
                              ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userGrantRole(user, role)
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/user/revoke_role")
    public void userRevokeRole(@RequestParam String sessionId,
                               @RequestParam String user,
                               @RequestParam String role,
                               ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .userRevokeRole(user, role)
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/role/list")
    public void listRoles(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleList()
                .whenComplete(((roles, throwable) -> handleEtcdComplete(future, roles, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/role/add")
    public void addRole(@RequestParam String sessionId,
                        @RequestParam String role,
                        ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleAdd(role)
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/role/delete")
    public void deleteRole(@RequestParam String sessionId,
                           @RequestParam String role,
                           ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleDel(role)
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/role/get_permissions")
    public void getRolePermissions(@RequestParam String sessionId,
                                   @RequestParam String role,
                                   ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .roleGet(role)
                .whenComplete(((permissions, throwable) -> handleEtcdComplete(future, permissions, throwable)));
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/etcd/role/grant_permission", method = Method.POST)
    public void roleGrantPermission(@RequestBody PermissionDTO data, ResponseFuture future) {
        EtcdConnectorFactory.get(data.getSessionId())
                .roleGrantPermission(data.getRole(), data.getKey(), data.parseRangeEnd(), data.getType())
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/etcd/role/revoke_permission", method = Method.POST)
    public void roleRevokePermission(@RequestBody PermissionDTO data, ResponseFuture future) {
        EtcdConnectorFactory.get(data.getSessionId())
                .roleRevokePermission(data.getRole(), data.getKey(), data.parseRangeEnd())
                .whenComplete(((response, throwable) -> handleEtcdComplete(future, null, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/cluster/get")
    public void getCluster(@RequestParam String sessionId, ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .clusterInfo()
                .whenComplete(((clusters, throwable) -> handleEtcdComplete(future, clusters, throwable)));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/cluster/remove_member")
    public void listClusterMember(@RequestParam String sessionId,
                                  @RequestParam String memberId,
                                  ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .clusterRemove(memberId)
                .whenComplete(((members, throwable) -> handleEtcdComplete(future, members, throwable)));
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/etcd/cluster/add_member", method = Method.POST)
    public void addClusterMember(@RequestBody MemberDTO member, ResponseFuture future) {
        List<String> urlList = member.getUrlList();
        if (urlList == null || urlList.isEmpty()) {
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
                .whenComplete(((memberBO, throwable) -> handleEtcdComplete(future, memberBO, throwable)));
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/etcd/cluster/update_member", method = Method.POST)
    public void updateClusterMember(@RequestBody MemberDTO member, ResponseFuture future) {
        List<String> urlList = member.getUrlList();
        if (urlList == null || urlList.isEmpty()) {
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
                .whenComplete((members, throwable) -> handleEtcdComplete(future, members, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/cluster/get_status")
    public void getMemberStatus(@RequestParam String sessionId,
                                @RequestParam String target,
                                ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .maintenanceMemberStatus(target)
                .whenComplete((memberStatusBO, throwable) -> handleEtcdComplete(future, memberStatusBO, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/cluster/defragment")
    public void defragment(@RequestParam String sessionId,
                           @RequestParam String target,
                           ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .maintenanceDefragment(target)
                .whenComplete((memberStatusBO, throwable) -> handleEtcdComplete(future, memberStatusBO, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/auth/enable")
    public void authEnable(@RequestParam String sessionId,
                           ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .authEnable()
                .whenComplete((resp, throwable) -> handleEtcdComplete(future, resp, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/auth/disable")
    public void authDisable(@RequestParam String sessionId,
                            ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .authDisable()
                .whenComplete((resp, throwable) -> handleEtcdComplete(future, resp, throwable));
    }

    @HttpRequest(Mapping.PRIVATE_API_PREFIX + "/session/etcd/export_keys")
    public void exportKeys(@RequestParam String sessionId,
                           @RequestParam String[] keys,
                           ResponseFuture future) {
        EtcdConnectorFactory.get(sessionId)
                .exportKeys(keys)
                .whenComplete((resp, throwable) -> handleEtcdComplete(future, resp, throwable));
    }

    @HttpRequest(value = Mapping.PRIVATE_API_PREFIX + "/session/etcd/import_keys", method = Method.POST)
    public void importKeys(@RequestBody ImportDTO data, ResponseFuture future) {
        EtcdConnectorFactory.get(data.getSessionId())
                .importKeys(data.getData())
                .whenComplete((resp, throwable) -> handleEtcdComplete(future, resp, throwable));
    }

    private void handleEtcdComplete(ResponseFuture future, Object result, Throwable throwable) {
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
                log.debug(throwable.getMessage(), throwable);
                future.apply(ResultCode.CONNECT_ERROR.result("Connect timeout", null));
            } else if (throwable instanceof IllegalArgumentException) {
                log.debug(throwable.getMessage(), throwable);
                future.apply(ResultCode.PARAM_FORMAT_ERROR.result(throwable.getMessage(), null));
            } else {
                if (!(throwable instanceof EtcdException)) {
                    log.error(throwable.getMessage(), throwable);
                }
                future.apply(ResultCode.ETCD_ERROR.result(throwable.getMessage(), null));
            }
        }
    }
}
