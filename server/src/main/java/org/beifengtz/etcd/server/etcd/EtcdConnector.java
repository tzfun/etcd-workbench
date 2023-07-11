package org.beifengtz.etcd.server.etcd;

import io.etcd.jetcd.Auth;
import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.KV;
import io.etcd.jetcd.KeyValue;
import io.etcd.jetcd.Response.Header;
import io.etcd.jetcd.auth.AuthRoleGetResponse;
import io.etcd.jetcd.auth.AuthRoleListResponse;
import io.etcd.jetcd.auth.AuthUserGetResponse;
import io.etcd.jetcd.auth.AuthUserListResponse;
import io.etcd.jetcd.auth.Permission;
import io.etcd.jetcd.cluster.Member;
import io.etcd.jetcd.cluster.MemberAddResponse;
import io.etcd.jetcd.cluster.MemberListResponse;
import io.etcd.jetcd.cluster.MemberRemoveResponse;
import io.etcd.jetcd.cluster.MemberUpdateResponse;
import io.etcd.jetcd.common.exception.ClosedClientException;
import io.etcd.jetcd.common.exception.EtcdException;
import io.etcd.jetcd.kv.DeleteResponse;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.kv.TxnResponse;
import io.etcd.jetcd.maintenance.AlarmMember;
import io.etcd.jetcd.maintenance.AlarmResponse;
import io.etcd.jetcd.maintenance.AlarmType;
import io.etcd.jetcd.maintenance.SnapshotResponse;
import io.etcd.jetcd.maintenance.StatusResponse;
import io.etcd.jetcd.op.Cmp;
import io.etcd.jetcd.op.CmpTarget;
import io.etcd.jetcd.op.Op;
import io.etcd.jetcd.options.DeleteOption;
import io.etcd.jetcd.options.GetOption;
import io.etcd.jetcd.options.PutOption;
import io.grpc.StatusRuntimeException;
import io.grpc.stub.StreamObserver;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.entity.bo.ClusterBO;
import org.beifengtz.etcd.server.entity.bo.KeyBO;
import org.beifengtz.etcd.server.entity.bo.KeyValueBO;
import org.beifengtz.etcd.server.entity.bo.MemberBO;
import org.beifengtz.etcd.server.entity.bo.PermissionBO;
import org.beifengtz.etcd.server.entity.bo.PermissionBO.PermissionBOBuilder;
import org.beifengtz.etcd.server.entity.bo.UserBO;
import org.beifengtz.etcd.server.exception.EtcdExecuteException;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.math.BigInteger;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Locale;
import java.util.UUID;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicInteger;
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

    public EtcdConnector(Client client) {
        this.client = client;
        this.connKey = UUID.randomUUID().toString().replaceAll("-", "").toLowerCase(Locale.ROOT);
        this.activeTime = System.currentTimeMillis();
        try {
            client.getKVClient()
                    .get(CommonUtil.toByteSequence(" "))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (ExecutionException e) {
            Throwable cause = e.getCause();
            if (cause instanceof ExecutionException) {
                cause = cause.getCause();
            }

            if (cause instanceof StatusRuntimeException) {
                throw new EtcdExecuteException(((StatusRuntimeException) cause).getStatus().getCode().name(), cause);
            } else {
                throw new EtcdExecuteException("Connect failed." + (e.getMessage() == null ? "" : (" " + e.getMessage())), e);
            }
        } catch (TimeoutException e) {
            throw new EtcdExecuteException("Connect timeout", e);
        } catch (InterruptedException e) {
            throw new EtcdExecuteException("Connect failed." + (e.getMessage() == null ? "" : (" " + e.getMessage())), e);
        }
    }

    public String getConnKey() {
        return connKey;
    }

    public void onActive() {
        activeTime = System.currentTimeMillis();
    }

    boolean checkRelease() {
        //  3分钟无响应，释放连接
        if (System.currentTimeMillis() - activeTime > TimeUnit.MINUTES.toMillis(3)) {
            client.close();
            logger.debug("Connector closed by factory monitor. {}", connKey);
            return true;
        }
        return false;
    }

    public void close() {
        client.close();
        EtcdConnectorFactory.onClose(connKey);
        logger.debug("Connector closed by invoke. {}", connKey);
    }

    private void onExecuteError(Throwable e) throws EtcdExecuteException {
        logger.error("Etcd client execute failed. " + e.getClass().getName() + ": " + e.getMessage(), e);
        if (e instanceof ClosedClientException) {
            close();
        }

        if (e.getCause() instanceof EtcdException) {
            Throwable cause = e.getCause();
            while (cause.getCause() != null) {
                cause = cause.getCause();
            }
            throw new EtcdExecuteException(cause.getMessage(), cause);
        }
        throw new EtcdExecuteException(e);
    }

    /**
     * 自定义获取键值对参数
     *
     * @param option 参数选项
     * @return List of {@link KeyValue}
     */
    public List<KeyValueBO> kvGet(String key, GetOption option) {
        onActive();
        try {
            GetResponse resp = client.getKVClient()
                    .get(CommonUtil.toByteSequence(key), option)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            List<KeyValue> kvs = resp.getKvs();
            List<KeyValueBO> res = new ArrayList<>(kvs.size());
            for (KeyValue kv : kvs) {
                res.add(KeyValueBO.parseFrom(kv));
            }
            return res;
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 获取键所对应的值，固定匹配
     *
     * @param key 键
     * @return 值
     */
    public KeyValueBO kvGet(String key) {
        List<KeyValueBO> kvs = kvGet(key, false);
        return kvs.size() > 0 ? kvs.get(0) : null;
    }

    /**
     * 获取键所对应的值，支持前缀匹配
     *
     * @param key      键
     * @param isPrefix 键是否是前缀匹配
     * @return Map 所有满足条件的键值对
     */
    public List<KeyValueBO> kvGet(String key, boolean isPrefix) {
        return kvGet(key, GetOption.newBuilder().isPrefix(isPrefix).build());
    }

    /**
     * 获取某一个key的所有历史版本
     *
     * @param key           key
     * @param startRevision 开始版本号
     * @param endRevision   结束版本号
     * @return 版本号列表
     */
    public Collection<Long> kvGetHistoryVersion(String key, long startRevision, long endRevision) {
        onActive();
        try {
            assert endRevision > startRevision;
            KV kvClient = client.getKVClient();
            ByteSequence key0 = CommonUtil.toByteSequence(key);
            List<Long> historyVersion = new ArrayList<>();
            long rev = endRevision;
            while (rev >= startRevision && rev <= endRevision) {
                GetResponse getResponse = kvClient.get(key0, GetOption.newBuilder()
                        .withRevision(rev)
                        .withKeysOnly(true)
                        .build()).get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
                if (getResponse.getCount() > 0) {
                    KeyValue kv = getResponse.getKvs().get(0);
                    if (rev >= kv.getCreateRevision() && rev <= kv.getModRevision()) {
                        historyVersion.add(rev);
                        rev--;
                    } else {
                        rev = kv.getModRevision();
                    }
                } else {
                    break;
                }
            }
            return historyVersion;
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return List.of();
    }

    /**
     * 获取所有key
     *
     * @return keys
     */
    public List<? extends KeyBO> kvGetAllKeys() {
        return kvGet(" ", GetOption.newBuilder()
                .isPrefix(true)
                .withRange(CommonUtil.toByteSequence("\0"))
                .withKeysOnly(true)
                .build());
    }

    /**
     * 设置键值对
     *
     * @param key   键
     * @param value 值
     */
    public void kvPut(String key, String value) {
        onActive();
        try {
            client.getKVClient()
                    .put(CommonUtil.toByteSequence(key), CommonUtil.toByteSequence(value))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * Compare And Swap
     *
     * @param key         key
     * @param expectValue 期待值
     * @param updateValue 更新值
     * @return 是否成功
     */
    public boolean kvCas(String key, String expectValue, String updateValue) {
        onActive();
        ByteSequence bsKey = CommonUtil.toByteSequence(key);
        ByteSequence bsExpectValue = CommonUtil.toByteSequence(expectValue);
        ByteSequence bsUpdateValue = CommonUtil.toByteSequence(updateValue);

        Cmp cmp = new Cmp(bsKey, Cmp.Op.EQUAL, CmpTarget.value(bsExpectValue));

        try {
            TxnResponse txnResponse = client.getKVClient()
                    .txn()
                    .If(cmp)
                    .Then(Op.put(bsKey, bsUpdateValue, PutOption.DEFAULT))
                    .commit()
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);

            return txnResponse.isSucceeded() && !txnResponse.getPutResponses().isEmpty();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return false;
    }

    /**
     * 删除一个键，固定匹配
     *
     * @param key 键
     * @return 成功条数
     */
    public long kvDel(String key) {
        return kvDel(key, false);
    }

    /**
     * 批量删除key，指定key，不按前缀删除
     *
     * @param keys 指定的key数组
     * @return 成功条数
     */
    public int kvDelBatch(String... keys) {
        onActive();
        try {
            KV kvClient = client.getKVClient();
            CountDownLatch cdl = new CountDownLatch(keys.length);
            AtomicInteger success = new AtomicInteger(0);
            for (String s : keys) {
                kvClient.delete(CommonUtil.toByteSequence(s), DeleteOption.newBuilder().isPrefix(false).build())
                        .whenCompleteAsync((deleteResponse, throwable) -> {
                            try {
                                if (throwable != null) {
                                    logger.error("Delete batch error", throwable);
                                } else {
                                    success.incrementAndGet();
                                }
                            } finally {
                                cdl.countDown();
                            }
                        });
            }
            cdl.await(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis() * 2L, TimeUnit.SECONDS);
            return success.get();
        } catch (InterruptedException e) {
            onExecuteError(e);
        }
        return 0;
    }

    /**
     * 删除一个或多个键
     *
     * @param key      键
     * @param isPrefix 是否是前缀匹配
     * @return 成功条数
     */
    public long kvDel(String key, boolean isPrefix) {
        onActive();
        try {
            DeleteResponse resp = client.getKVClient()
                    .delete(CommonUtil.toByteSequence(key), DeleteOption.newBuilder().isPrefix(isPrefix).build())
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return resp.getDeleted();
        } catch (InterruptedException | ExecutionException | TimeoutException e) {
            onExecuteError(e);
        }
        return 0;
    }

    /**
     * 获取所有用户
     *
     * @return List of user
     */
    public List<String> userList() {
        onActive();
        try {
            AuthUserListResponse userList = client.getAuthClient()
                    .userList()
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return userList.getUsers();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    public List<UserBO> userFullList() {
        onActive();
        try {
            Auth authClient = client.getAuthClient();
            AuthUserListResponse userList = authClient
                    .userList()
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            int count = userList.getUsers().size();
            if (count == 0) {
                return List.of();
            }
            List<UserBO> result = new ArrayList<>(count);
            CountDownLatch cdl = new CountDownLatch(count);
            for (String user : userList.getUsers()) {
                UserBO userBO = new UserBO();
                userBO.setUser(user);
                result.add(userBO);
                authClient.userGet(CommonUtil.toByteSequence(user)).whenCompleteAsync(((authUserGetResponse, throwable) -> {
                    try {
                        if (throwable != null) {
                            logger.error("Query user role failed ", throwable);
                        } else {
                            userBO.setRoles(authUserGetResponse.getRoles());
                        }
                    } finally {
                        cdl.countDown();
                    }
                }));
            }
            cdl.await(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis() * 2L, TimeUnit.SECONDS);
            return result;
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return List.of();
    }

    /**
     * 获取某个用户的角色信息
     *
     * @param user 用户名
     * @return List of role
     */
    public List<String> userGetRoles(String user) {
        onActive();
        try {
            AuthUserGetResponse userGet = client.getAuthClient()
                    .userGet(CommonUtil.toByteSequence(user))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return userGet.getRoles();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return List.of();
    }

    /**
     * 添加用户
     *
     * @param user     用户名
     * @param password 密码
     */
    public void userAdd(String user, String password) {
        onActive();
        try {
            client.getAuthClient()
                    .userAdd(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(password))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 删除用户
     *
     * @param user 用户名
     */
    public void userDel(String user) {
        onActive();
        try {
            client.getAuthClient()
                    .userDelete(CommonUtil.toByteSequence(user))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 修改用户密码
     *
     * @param user        用户名
     * @param newPassword 新密码
     */
    public void userChangePassword(String user, String newPassword) {
        onActive();
        try {
            client.getAuthClient()
                    .userChangePassword(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(newPassword))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 给用户授权一个角色
     *
     * @param user 用户名
     * @param role 角色名
     */
    public void userGrantRole(String user, String role) {
        onActive();
        try {
            client.getAuthClient()
                    .userGrantRole(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(role))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 回收用户一个角色
     *
     * @param user 用户名
     * @param role 角色名
     */
    public void userRevokeRole(String user, String role) {
        onActive();
        try {
            client.getAuthClient()
                    .userRevokeRole(CommonUtil.toByteSequence(user), CommonUtil.toByteSequence(role))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 获取角色权限
     *
     * @param role 角色
     * @return List of {@link Permission}
     */
    public List<PermissionBO> roleGet(String role) {
        onActive();
        try {
            AuthRoleGetResponse roleGet = client.getAuthClient()
                    .roleGet(CommonUtil.toByteSequence(role))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            List<Permission> permissions = roleGet.getPermissions();
            if (permissions.size() == 0) {
                return List.of();
            }
            List<PermissionBO> result = new ArrayList<>();
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
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return List.of();
    }

    /**
     * 获取所有角色
     *
     * @return List of role
     */
    public List<String> roleList() {
        onActive();
        try {
            AuthRoleListResponse roleList = client.getAuthClient()
                    .roleList()
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return roleList.getRoles();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return List.of();
    }

    /**
     * 添加一个角色
     *
     * @param role 角色名
     */
    public void roleAdd(String role) {
        onActive();
        try {
            client.getAuthClient()
                    .roleAdd(CommonUtil.toByteSequence(role))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 删除一个角色
     *
     * @param role 角色名
     */
    public void roleDel(String role) {
        onActive();
        try {
            client.getAuthClient()
                    .roleDelete(CommonUtil.toByteSequence(role))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 给角色授权访问权限
     *
     * @param role       角色名
     * @param key        key
     * @param rangeEnd   权限范围结束限制
     * @param permission 权限类型
     */
    public void roleGrantPermission(String role, String key, ByteSequence rangeEnd, Permission.Type permission) {
        onActive();
        try {
            client.getAuthClient()
                    .roleGrantPermission(CommonUtil.toByteSequence(role), CommonUtil.toByteSequence(key), rangeEnd, permission)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 回收角色的权限
     *
     * @param role     角色
     * @param key      key
     * @param rangeEnd 权限结束标识符
     */
    public void roleRevokePermission(String role, String key, ByteSequence rangeEnd) {
        onActive();
        try {
            client.getAuthClient()
                    .roleRevokePermission(CommonUtil.toByteSequence(role), CommonUtil.toByteSequence(key), rangeEnd)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 获取集群所有节点
     *
     * @return 节点列表
     */
    public ClusterBO clusterInfo() {
        onActive();
        try {
            MemberListResponse memberListResponse = client.getClusterClient()
                    .listMember()
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            Header header = memberListResponse.getHeader();

            ClusterBO cluster = new ClusterBO();
            cluster.setClusterId(Long.toUnsignedString(header.getClusterId()));
            cluster.setLeaderId(Long.toUnsignedString(header.getMemberId()));
            cluster.setRevision(header.getRevision());
            cluster.setRaftTerm(header.getRaftTerm());
            List<Member> memberList = memberListResponse.getMembers();
            if (memberList.size() > 0) {
                cluster.setMembers(memberList.stream().map(MemberBO::parseFrom).collect(Collectors.toList()));
            }
            return cluster;
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 从集群中移除一个节点
     *
     * @param memberId 成员节点ID
     * @return 移除后集群的节点列表
     */
    public List<MemberBO> clusterRemove(String memberId) {
        onActive();
        try {
            MemberRemoveResponse memberRemove = client.getClusterClient()
                    .removeMember(new BigInteger(memberId).longValue())
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            List<Member> members = memberRemove.getMembers();
            if (members == null || members.size() == 0) {
                return List.of();
            }
            return members.stream().map(MemberBO::parseFrom).collect(Collectors.toList());
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 向集群中添加一个节点
     *
     * @param urls 节点地址，集群通过此地址与之通信
     * @return 添加的节点信息
     */
    public MemberBO clusterAdd(List<URI> urls) {
        onActive();
        try {
            MemberAddResponse memberAdd = client.getClusterClient()
                    .addMember(urls)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return MemberBO.parseFrom(memberAdd.getMember());
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 更新集群中一个节点信息
     *
     * @param memberId 节点ID
     * @param urls     节点地址，集群通过此地址与之通信
     * @return 更新的节点列表
     */
    public List<Member> clusterUpdate(String memberId, List<URI> urls) {
        onActive();
        try {
            MemberUpdateResponse memberUpdate = client.getClusterClient()
                    .updateMember(new BigInteger(memberId).longValue(), urls)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return memberUpdate.getMembers();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 运维相关接口，获取所有报警信息
     *
     * @return List of {@link AlarmMember}
     */
    public List<AlarmMember> maintenanceAlarmList() {
        onActive();
        try {
            AlarmResponse alarmResponse = client.getMaintenanceClient()
                    .listAlarms()
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return alarmResponse.getAlarms();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 消除一个警报
     *
     * @param memberId 节点ID
     * @param type     警报类型
     * @return 剩余的警报
     */
    public List<AlarmMember> maintenanceAlarmDisarm(long memberId, AlarmType type) {
        onActive();
        try {
            AlarmResponse alarmResponse = client.getMaintenanceClient()
                    .alarmDisarm(new AlarmMember(memberId, type))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return alarmResponse.getAlarms();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 获取节点的状态信息
     *
     * @param target 目标端口 endpoints，也可以是Peer URL
     * @return {@link StatusResponse}
     */
    public StatusResponse maintenanceMemberStatus(String target) {
        onActive();
        try {
            return client.getMaintenanceClient()
                    .statusMember(target)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 对节点进行碎片清理。这是一个比较消耗资源的操作，谨慎调用。
     *
     * @param target 目标端口endpoints，也可以是Peer URL
     */
    public void maintenanceGc(String target) {
        onActive();
        try {
            client.getMaintenanceClient().defragmentMember(target)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }

    /**
     * 备份当前快照
     *
     * @param observer 监视器，当备份完毕后会调用其中的Next方法
     */
    public void maintenanceSnapshot(StreamObserver<SnapshotResponse> observer) {
        onActive();
        try {
            client.getMaintenanceClient().snapshot(observer);
        } catch (Throwable e) {
            onExecuteError(e);
        }
    }
}
