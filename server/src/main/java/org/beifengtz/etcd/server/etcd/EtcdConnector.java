package org.beifengtz.etcd.server.etcd;

import io.etcd.jetcd.Auth;
import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.KV;
import io.etcd.jetcd.KeyValue;
import io.etcd.jetcd.Response.Header;
import io.etcd.jetcd.auth.AuthDisableResponse;
import io.etcd.jetcd.auth.AuthEnableResponse;
import io.etcd.jetcd.auth.AuthRoleAddResponse;
import io.etcd.jetcd.auth.AuthRoleDeleteResponse;
import io.etcd.jetcd.auth.AuthRoleGrantPermissionResponse;
import io.etcd.jetcd.auth.AuthRoleListResponse;
import io.etcd.jetcd.auth.AuthRoleRevokePermissionResponse;
import io.etcd.jetcd.auth.AuthUserAddResponse;
import io.etcd.jetcd.auth.AuthUserChangePasswordResponse;
import io.etcd.jetcd.auth.AuthUserDeleteResponse;
import io.etcd.jetcd.auth.AuthUserGetResponse;
import io.etcd.jetcd.auth.AuthUserGrantRoleResponse;
import io.etcd.jetcd.auth.AuthUserRevokeRoleResponse;
import io.etcd.jetcd.auth.Permission;
import io.etcd.jetcd.cluster.Member;
import io.etcd.jetcd.cluster.MemberUpdateResponse;
import io.etcd.jetcd.kv.DeleteResponse;
import io.etcd.jetcd.maintenance.AlarmMember;
import io.etcd.jetcd.maintenance.AlarmResponse;
import io.etcd.jetcd.maintenance.AlarmType;
import io.etcd.jetcd.maintenance.DefragmentResponse;
import io.etcd.jetcd.maintenance.SnapshotResponse;
import io.etcd.jetcd.options.DeleteOption;
import io.etcd.jetcd.options.GetOption;
import io.etcd.jetcd.options.PutOption;
import io.grpc.stub.StreamObserver;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.entity.SshContext;
import org.beifengtz.etcd.server.entity.bo.ClusterBO;
import org.beifengtz.etcd.server.entity.bo.KeyValueBO;
import org.beifengtz.etcd.server.entity.bo.MemberBO;
import org.beifengtz.etcd.server.entity.bo.MemberStatusBO;
import org.beifengtz.etcd.server.entity.bo.PermissionBO;
import org.beifengtz.etcd.server.entity.bo.PermissionBO.PermissionBOBuilder;
import org.beifengtz.etcd.server.entity.bo.UserBO;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.math.BigInteger;
import java.net.URI;
import java.net.URISyntaxException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentSkipListSet;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import java.util.stream.Collectors;

/**
 * description: TODO
 * date: 16:10 2023/5/23
 *
 * @author beifengtz
 */
public class EtcdConnector {

    private static final Logger logger = LoggerFactory.getLogger(EtcdConnector.class);

    private final String connKey;
    private final Client client;
    private long activeTime;
    private final SshContext sshContext;

    public EtcdConnector(Client client, SshContext sshContext) {
        this.client = client;
        this.connKey = UUID.randomUUID().toString().replaceAll("-", "").toLowerCase(Locale.ROOT);
        this.activeTime = System.currentTimeMillis();
        this.sshContext = sshContext;
    }

    public String getConnKey() {
        return connKey;
    }

    public void onActive() {
        activeTime = System.currentTimeMillis();
    }

    boolean checkRelease() {
        //  35秒无响应，释放连接
        if (System.currentTimeMillis() - activeTime > TimeUnit.SECONDS.toMillis(35)) {
            client.close();
            logger.debug("Connector closed by factory monitor. {}", connKey);
            return true;
        }
        return false;
    }

    public void close() {
        client.close();
        if (sshContext != null) {
            sshContext.getSession().disconnect();
        }
        EtcdConnectorFactory.onClose(connKey);
        logger.debug("Connector closed by invoke. {}", connKey);
    }

    private String transferTarget(String target) {
        if (sshContext != null) {
            try {
                URI uri = new URI(target);
                String parsedTarget = uri.getScheme() + "://";
                if (Objects.equals(uri.getHost(), sshContext.getSrcHost())) {
                    parsedTarget += sshContext.getProxyLocalHost();
                } else {
                    parsedTarget += uri.getHost();
                }
                parsedTarget += ":";
                if (uri.getPort() == sshContext.getSrcPort()) {
                    parsedTarget += sshContext.getProxyLocalPort();
                } else {
                    parsedTarget += uri.getPort();
                }
                return parsedTarget;
            } catch (URISyntaxException e) {
                logger.warn("Parse target failed: " + e.getMessage(), e);
                return target;
            }
        }
        return target;
    }

    /**
     * 自定义获取键值对参数
     *
     * @param option 参数选项
     * @return List of {@link KeyValue}
     */
    public CompletableFuture<List<KeyValueBO>> kvGet(String key, GetOption option) {
        onActive();
        return client.getKVClient()
                .get(CommonUtil.toByteSequence(key), option)
                .thenApply(resp -> {
                    List<KeyValue> kvs = resp.getKvs();
                    List<KeyValueBO> res = new ArrayList<>(kvs.size());
                    for (KeyValue kv : kvs) {
                        res.add(KeyValueBO.parseFrom(kv));
                    }
                    return res;
                })
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 获取键所对应的值，固定匹配
     *
     * @param key 键
     * @return 值
     */
    public CompletableFuture<KeyValueBO> kvGet(String key) {
        return kvGet(key, false)
                .thenApply(kvs -> !kvs.isEmpty() ? kvs.get(0) : null);
    }

    /**
     * 获取键所对应的值，支持前缀匹配
     *
     * @param key      键
     * @param isPrefix 键是否是前缀匹配
     * @return Map 所有满足条件的键值对
     */
    public CompletableFuture<List<KeyValueBO>> kvGet(String key, boolean isPrefix) {
        return kvGet(key, isPrefix
                ? GetOption.newBuilder().isPrefix(true).build()
                : GetOption.DEFAULT)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 获取某一个key的所有历史版本
     *
     * @param key           key
     * @param startRevision 开始版本号
     * @param endRevision   结束版本号
     * @return 版本号列表
     */
    public CompletableFuture<Collection<Long>> kvGetHistoryVersion(String key, long startRevision, long endRevision) {
        onActive();
        CompletableFuture<Collection<Long>> future = new CompletableFuture<>();
        assert endRevision > startRevision;
        KV kvClient = client.getKVClient();
        ByteSequence keyByte = CommonUtil.toByteSequence(key);
        Set<Long> historyVersion = new ConcurrentSkipListSet<>();
        AtomicLong rev = new AtomicLong(endRevision);

        Runnable search = new Runnable() {
            @Override
            public void run() {
                long v = rev.get();
                if (v >= startRevision && v <= endRevision) {
                    kvClient.get(keyByte, GetOption.newBuilder()
                                    .withRevision(v)
                                    .withKeysOnly(true)
                                    .build())
                            .whenComplete((getResponse, throwable) -> {
                                if (throwable == null) {
                                    if (getResponse.getCount() > 0) {
                                        KeyValue kv = getResponse.getKvs().get(0);
                                        if (v >= kv.getCreateRevision() && v <= kv.getModRevision()) {
                                            historyVersion.add(v);
                                            rev.decrementAndGet();
                                        } else {
                                            rev.set(kv.getModRevision());
                                        }
                                        this.run();
                                    } else {
                                        future.complete(historyVersion);
                                    }
                                } else {
                                    future.completeExceptionally(throwable);
                                }
                            });
                } else {
                    future.complete(historyVersion);
                }
            }
        };
        search.run();
        return future;
    }

    /**
     * 获取所有key
     *
     * @return keys
     */
    public CompletableFuture<List<KeyValueBO>> kvGetAllKeys() {
        return kvGet("\0", GetOption.newBuilder()
                .isPrefix(true)
                .withRange(CommonUtil.toByteSequence("\0"))
                .withKeysOnly(true)
                .build())
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 设置键值对
     *
     * @param key   键
     * @param value 值
     */
    public CompletableFuture<?> kvPut(String key, String value) {
        return kvPut(key, value, null);
    }

    /**
     * 设置键值对
     *
     * @param key   键
     * @param value 值
     * @param ttl   失效时间，如果为null则表示永不失效，单位秒
     * @return {@link CompletableFuture}
     */
    public CompletableFuture<?> kvPut(String key, String value, Long ttl) {
        onActive();
        if (ttl == null) {
            ByteSequence byteKey = CommonUtil.toByteSequence(key);
            return client.getKVClient()
                    .get(byteKey, GetOption.newBuilder().withKeysOnly(true).build())
                    .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS)
                    .thenCompose(resp -> {
                        long lease = 0;
                        if (resp.getCount() == 1) {
                            lease = resp.getKvs().get(0).getLease();
                        }
                        if (lease != 0) {
                            return client.getKVClient()
                                    .put(byteKey, CommonUtil.toByteSequence(value), PutOption.newBuilder().withLeaseId(lease).build())
                                    .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
                        } else {
                            return client.getKVClient()
                                    .put(byteKey, CommonUtil.toByteSequence(value))
                                    .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
                        }
                    });
        } else {
            return client.getLeaseClient().grant(ttl).thenCompose(r -> {
                long leaseId = r.getID();
                PutOption putOption = PutOption.newBuilder().withLeaseId(leaseId).build();
                return client.getKVClient()
                        .put(CommonUtil.toByteSequence(key), CommonUtil.toByteSequence(value), putOption)
                        .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            });

        }
    }

    /**
     * 删除一个键，固定匹配
     *
     * @param key 键
     * @return 成功条数
     */
    public CompletableFuture<Long> kvDel(String key) {
        return kvDel(key, false);
    }

    /**
     * 批量删除key，指定key，不按前缀删除
     *
     * @param keys 指定的key数组
     * @return 成功条数
     */
    public CompletableFuture<Integer> kvDelBatch(String... keys) {
        onActive();
        KV kvClient = client.getKVClient();
        CompletableFuture<Integer> future = new CompletableFuture<>();

        AtomicInteger success = new AtomicInteger(0);
        AtomicInteger counter = new AtomicInteger(0);
        for (String s : keys) {
            kvClient.delete(CommonUtil.toByteSequence(s), DeleteOption.newBuilder().isPrefix(false).build())
                    .whenComplete((deleteResponse, throwable) -> {
                        try {
                            if (throwable == null) {
                                success.incrementAndGet();
                            } else {
                                logger.error("Delete batch error", throwable);
                            }
                        } finally {
                            if (counter.incrementAndGet() >= keys.length) {
                                future.complete(success.get());
                            }
                        }
                    });
        }

        return future;
    }

    /**
     * 删除一个或多个键
     *
     * @param key      键
     * @param isPrefix 是否是前缀匹配
     * @return 成功条数
     */
    public CompletableFuture<Long> kvDel(String key, boolean isPrefix) {
        onActive();
        return client.getKVClient()
                .delete(CommonUtil.toByteSequence(key), DeleteOption.newBuilder().isPrefix(isPrefix).build())
                .thenApply(DeleteResponse::getDeleted)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    public CompletableFuture<Collection<UserBO>> userFullList() {
        onActive();
        Set<UserBO> set = ConcurrentHashMap.newKeySet();

        Auth authClient = client.getAuthClient();
        CompletableFuture<Collection<UserBO>> future = new CompletableFuture<>();

        authClient.userList().whenComplete((authUserListResponse, throwable) -> {
            if (throwable == null) {
                List<String> users = authUserListResponse.getUsers();
                int count = users.size();
                if (count > 0) {
                    AtomicInteger counter = new AtomicInteger(0);
                    for (String user : users) {
                        if (future.isDone()) {
                            break;
                        }
                        UserBO userBO = new UserBO();
                        userBO.setUser(user);
                        set.add(userBO);
                        authClient.userGet(CommonUtil.toByteSequence(user)).whenComplete(((authUserGetResponse, t) -> {
                            if (t == null) {
                                userBO.setRoles(authUserGetResponse.getRoles());
                                if (counter.incrementAndGet() >= count) {
                                    future.complete(set);
                                }
                            } else {
                                future.completeExceptionally(t);
                            }
                        }));
                    }
                } else {
                    future.complete(set);
                }
            } else {
                future.completeExceptionally(throwable);
            }
        });
        return future;
    }

    /**
     * 获取某个用户的角色信息
     *
     * @param user 用户名
     * @return List of role
     */
    public CompletableFuture<List<String>> userGetRoles(String user) {
        onActive();
        return client.getAuthClient()
                .userGet(CommonUtil.toByteSequence(user))
                .thenApply(AuthUserGetResponse::getRoles)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 添加用户
     *
     * @param user     用户名
     * @param password 密码
     */
    public CompletableFuture<AuthUserAddResponse> userAdd(String user, String password) {
        onActive();
        return client.getAuthClient()
                .userAdd(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(password))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 删除用户
     *
     * @param user 用户名
     */
    public CompletableFuture<AuthUserDeleteResponse> userDel(String user) {
        onActive();
        return client.getAuthClient()
                .userDelete(CommonUtil.toByteSequence(user))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 修改用户密码
     *
     * @param user        用户名
     * @param newPassword 新密码
     */
    public CompletableFuture<AuthUserChangePasswordResponse> userChangePassword(String user, String newPassword) {
        onActive();
        return client.getAuthClient()
                .userChangePassword(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(newPassword))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 给用户授权一个角色
     *
     * @param user 用户名
     * @param role 角色名
     */
    public CompletableFuture<AuthUserGrantRoleResponse> userGrantRole(String user, String role) {
        onActive();
        return client.getAuthClient()
                .userGrantRole(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(role))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 回收用户一个角色
     *
     * @param user 用户名
     * @param role 角色名
     */
    public CompletableFuture<AuthUserRevokeRoleResponse> userRevokeRole(String user, String role) {
        onActive();
        return client.getAuthClient()
                .userRevokeRole(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(role))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 判断此用户是否拥有root权限
     *
     * @param user ETCD用户
     * @return boolean future
     */
    @SuppressWarnings("unchecked")
    public CompletableFuture<Boolean> userIsRoot(String user) {
        if (user == null || "root".equals(user)) {
            return CompletableFuture.completedFuture(true);
        }

        return userGetRoles(user).thenCompose((roles) -> {
            if (roles.contains("root")) {
                return CompletableFuture.completedFuture(true);
            } else {
                CompletableFuture<Boolean>[] futures = new CompletableFuture[roles.size()];
                for (int i = 0; i < roles.size(); i++) {
                    futures[i] = roleGet(roles.get(i)).thenApply(permissions -> {
                        for (PermissionBO permission : permissions) {
                            if ("root".equals(permission.getKey())) {
                                return true;
                            }
                        }
                        return false;
                    });
                }

                return CompletableFuture.allOf(futures).thenApply(v -> {
                    for (CompletableFuture<Boolean> future : futures) {
                        if (future.getNow(false)) {
                            return true;
                        }
                    }
                    return false;
                });
            }
        });
    }

    /**
     * 获取角色权限
     *
     * @param role 角色
     * @return List of {@link Permission}
     */
    public CompletableFuture<List<PermissionBO>> roleGet(String role) {
        onActive();
        return client.getAuthClient()
                .roleGet(CommonUtil.toByteSequence(role))
                .thenApply(roleGet -> {
                    List<Permission> permissions = roleGet.getPermissions();
                    int count = permissions.size();
                    if (count == 0) {
                        return List.of();
                    } else {
                        List<PermissionBO> result = new ArrayList<>(count);
                        for (Permission permission : permissions) {
                            ByteSequence key = permission.getKey();
                            byte[] keyBytes = key.getBytes();
                            ByteSequence rangeEnd = permission.getRangeEnd();
                            byte[] rangeEndBytes = rangeEnd.getBytes();

                            PermissionBOBuilder<?, ?> builder = PermissionBO.builder().type(permission.getPermType());
                            //  为兼容老版本的etcd，空字符串是一个长度为1且内容为0的byte数组
                            boolean allKeys = (keyBytes.length == 0 && rangeEndBytes.length == 0) ||
                                    (keyBytes.length == 1 && rangeEndBytes.length == 1 && keyBytes[0] == 0 && rangeEndBytes[0] == 0);
                            if (allKeys) {
                                builder.allKeys(true);
                            } else {
                                if (rangeEndBytes.length >= keyBytes.length) {
                                    boolean prefixEqual = true;
                                    for (int i = 0; i < keyBytes.length - 1; i++) {
                                        if (keyBytes[i] != rangeEndBytes[i]) {
                                            prefixEqual = false;
                                            break;
                                        }
                                    }

                                    boolean prefix = false;

                                    //  判断是否是前缀匹配
                                    if (prefixEqual) {
                                        if (rangeEndBytes.length - keyBytes.length == 1) {
                                            prefix = rangeEndBytes[rangeEndBytes.length - 1] == 1;
                                        } else if (rangeEndBytes.length == keyBytes.length) {
                                            prefix = rangeEndBytes[rangeEndBytes.length - 1] - keyBytes[keyBytes.length - 1] == 1;
                                        }
                                    }

                                    builder.prefix(prefix);
                                }

                                builder.key(key.toString(StandardCharsets.UTF_8));
                            }

                            result.add(builder.build());
                        }
                        return result;
                    }
                });
    }

    /**
     * 获取所有角色
     *
     * @return List of role
     */
    public CompletableFuture<List<String>> roleList() {
        onActive();
        return client.getAuthClient()
                .roleList()
                .thenApply(AuthRoleListResponse::getRoles)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 添加一个角色
     *
     * @param role 角色名
     */
    public CompletableFuture<AuthRoleAddResponse> roleAdd(String role) {
        onActive();
        return client.getAuthClient()
                .roleAdd(CommonUtil.toByteSequence(role))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 删除一个角色
     *
     * @param role 角色名
     */
    public CompletableFuture<AuthRoleDeleteResponse> roleDel(String role) {
        onActive();
        return client.getAuthClient()
                .roleDelete(CommonUtil.toByteSequence(role))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 给角色授权访问权限
     *
     * @param role       角色名
     * @param key        key
     * @param rangeEnd   权限范围结束限制
     * @param permission 权限类型
     */
    public CompletableFuture<AuthRoleGrantPermissionResponse> roleGrantPermission(String role, String key, ByteSequence rangeEnd, Permission.Type permission) {
        onActive();
        return client.getAuthClient()
                .roleGrantPermission(CommonUtil.toByteSequence(role), CommonUtil.toByteSequence(key), rangeEnd, permission)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 回收角色的权限
     *
     * @param role     角色
     * @param key      key
     * @param rangeEnd 权限结束标识符
     */
    public CompletableFuture<AuthRoleRevokePermissionResponse> roleRevokePermission(String role, String key, ByteSequence rangeEnd) {
        onActive();
        return client.getAuthClient()
                .roleRevokePermission(CommonUtil.toByteSequence(role), CommonUtil.toByteSequence(key), rangeEnd)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 获取集群所有节点
     *
     * @return 节点列表
     */
    public CompletableFuture<ClusterBO> clusterInfo() {
        onActive();
        return client.getClusterClient().listMember().thenApply(memberListResponse -> {
            Header header = memberListResponse.getHeader();

            ClusterBO cluster = new ClusterBO();
            cluster.setClusterId(Long.toUnsignedString(header.getClusterId()));
            cluster.setLeaderId(Long.toUnsignedString(header.getMemberId()));
            cluster.setRevision(header.getRevision());
            cluster.setRaftTerm(header.getRaftTerm());
            List<Member> memberList = memberListResponse.getMembers();
            if (!memberList.isEmpty()) {
                cluster.setMembers(memberList.stream().map(MemberBO::parseFrom).collect(Collectors.toList()));
            }
            return cluster;
        }).orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 从集群中移除一个节点
     *
     * @param memberId 成员节点ID
     * @return 移除后集群的节点列表
     */
    public CompletableFuture<List<MemberBO>> clusterRemove(String memberId) {
        onActive();
        return client.getClusterClient()
                .removeMember(new BigInteger(memberId).longValue())
                .thenApply(memberRemove -> {
                    List<Member> members = memberRemove.getMembers();
                    if (members == null || members.isEmpty()) {
                        return List.of();
                    }
                    return members.stream().map(MemberBO::parseFrom).collect(Collectors.toList());
                });
    }

    /**
     * 向集群中添加一个节点
     *
     * @param urls 节点地址，集群通过此地址与之通信
     * @return 添加的节点信息
     */
    public CompletableFuture<MemberBO> clusterAdd(List<URI> urls) {
        onActive();
        return client.getClusterClient()
                .addMember(urls).thenApply(memberAdd -> MemberBO.parseFrom(memberAdd.getMember()))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 更新集群中一个节点信息
     *
     * @param memberId 节点ID
     * @param urls     节点地址，集群通过此地址与之通信
     * @return 更新的节点列表
     */
    public CompletableFuture<List<Member>> clusterUpdate(String memberId, List<URI> urls) {
        onActive();
        return client.getClusterClient()
                .updateMember(new BigInteger(memberId).longValue(), urls)
                .thenApply(MemberUpdateResponse::getMembers)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 运维相关接口，获取所有报警信息
     *
     * @return List of {@link AlarmMember}
     */
    public CompletableFuture<List<AlarmMember>> maintenanceAlarmList() {
        onActive();
        return client.getMaintenanceClient()
                .listAlarms()
                .thenApply(AlarmResponse::getAlarms)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 消除一个警报
     *
     * @param memberId 节点ID
     * @param type     警报类型
     * @return 剩余的警报
     */
    public CompletableFuture<List<AlarmMember>> maintenanceAlarmDisarm(long memberId, AlarmType type) {
        onActive();
        return client.getMaintenanceClient()
                .alarmDisarm(new AlarmMember(memberId, type))
                .thenApply(AlarmResponse::getAlarms)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 获取节点的状态信息
     *
     * @param target 目标 endpoints，也可以是 URL
     * @return {@link MemberStatusBO}
     */
    public CompletableFuture<MemberStatusBO> maintenanceMemberStatus(String target) {
        onActive();
        return client.getMaintenanceClient()
                .statusMember(transferTarget(target))
                .thenApply(MemberStatusBO::parseFrom)
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 对节点进行碎片清理。这是一个比较消耗资源的操作，谨慎调用。
     *
     * @param target 目标 endpoints，也可以是 URL
     */
    public CompletableFuture<DefragmentResponse> maintenanceGc(String target) {
        onActive();
        return client.getMaintenanceClient()
                .defragmentMember(transferTarget(target))
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 备份当前快照
     *
     * @param observer 监视器，当备份完毕后会调用其中的Next方法
     */
    public void maintenanceSnapshot(StreamObserver<SnapshotResponse> observer) {
        onActive();
        client.getMaintenanceClient().snapshot(observer);
    }

    /**
     * 开启认证
     */
    public CompletableFuture<AuthEnableResponse> authEnable() {
        onActive();
        return client.getAuthClient()
                .authEnable()
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }

    /**
     * 关闭认证
     */
    public CompletableFuture<AuthDisableResponse> authDisable() {
        return client.getAuthClient()
                .authDisable()
                .orTimeout(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
    }
}
