package org.beifengtz.etcd.server.etcd;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.KeyValue;
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
import io.etcd.jetcd.kv.DeleteResponse;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.kv.PutResponse;
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
import io.grpc.stub.StreamObserver;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.exception.EtcdExecuteException;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.net.URI;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static java.nio.charset.StandardCharsets.UTF_8;

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
        client.close();
    }

    public String getConnKey() {
        return connKey;
    }

    private void onActive() {
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
        throw new EtcdExecuteException(e);
    }

    /**
     * 自定义获取键值对参数
     *
     * @param option 参数选项
     * @return List of {@link KeyValue}
     */
    public List<KeyValue> kvGet(GetOption option) {
        onActive();
        try {
            GetResponse resp = client.getKVClient()
                    .get(CommonUtil.toByteSequence(connKey), option)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return resp.getKvs();
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
    public String kvGet(String key) {
        return kvGet(key, false).get(key);
    }

    /**
     * 获取键所对应的值，支持前缀匹配
     *
     * @param key      键
     * @param isPrefix 键是否是前缀匹配
     * @return Map 所有满足条件的键值对
     */
    public Map<String, String> kvGet(String key, boolean isPrefix) {
        onActive();
        try {
            GetResponse resp = client.getKVClient().get(CommonUtil.toByteSequence(key), GetOption.newBuilder().isPrefix(isPrefix).build())
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            List<KeyValue> kvs = resp.getKvs();
            Map<String, String> res = new HashMap<>();
            for (KeyValue kv : kvs) {
                res.put(kv.getKey().toString(UTF_8), kv.getValue().toString(UTF_8));
            }
            return res;
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 获取有绑定租约的键值对
     *
     * @return List of {@link KeyValue}
     */
    public List<KeyValue> kvGetLease() {
        onActive();
        try {
            GetResponse resp = client.getKVClient()
                    .get(ByteSequence.EMPTY, GetOption.newBuilder().isPrefix(true).build())
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            List<KeyValue> kvs = resp.getKvs();
            List<KeyValue> res = new ArrayList<>();
            for (KeyValue kv : kvs) {
                if (kv.getLease() != 0) {
                    res.add(kv);
                }
            }
            return res;
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
    }

    /**
     * 设置键值对
     *
     * @param key   键
     * @param value 值
     * @return 返回设置值之前的值
     */
    public String kvPut(String key, String value) {
        onActive();
        try {
            PutResponse resp = client.getKVClient()
                    .put(CommonUtil.toByteSequence(key), CommonUtil.toByteSequence(value))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            if (resp.hasPrevKv()) {
                return resp.getPrevKv().getValue().toString(UTF_8);
            }
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
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
        return null;
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
    public List<Permission> roleGet(String role) {
        onActive();
        try {
            AuthRoleGetResponse roleGet = client.getAuthClient()
                    .roleGet(CommonUtil.toByteSequence(role))
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return roleGet.getPermissions();
        } catch (Throwable e) {
            onExecuteError(e);
        }
        return null;
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
        return null;
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
    public void roleGrantPermission(String role, String key, String rangeEnd, Permission.Type permission) {
        onActive();
        try {
            client.getAuthClient()
                    .roleGrantPermission(CommonUtil.toByteSequence(role), CommonUtil.toByteSequence(key), CommonUtil.toByteSequence(rangeEnd), permission)
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
     * @param rangeEnd 权限范围结束限制
     */
    public void roleRevokePermission(String role, String key, String rangeEnd) {
        onActive();
        try {
            client.getAuthClient()
                    .roleRevokePermission(CommonUtil.toByteSequence(role), CommonUtil.toByteSequence(key), CommonUtil.toByteSequence(rangeEnd))
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
    public List<Member> clusterList() {
        onActive();
        try {
            MemberListResponse memberList = client.getClusterClient()
                    .listMember()
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return memberList.getMembers();
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
    public List<Member> clusterRemove(long memberId) {
        onActive();
        try {
            MemberRemoveResponse memberRemove = client.getClusterClient()
                    .removeMember(memberId)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return memberRemove.getMembers();
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
    public Member clusterAdd(List<URI> urls) {
        onActive();
        try {
            MemberAddResponse memberAdd = client.getClusterClient()
                    .addMember(urls)
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return memberAdd.getMember();
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
    public List<Member> clusterUpdate(long memberId, List<URI> urls) {
        onActive();
        try {
            MemberUpdateResponse memberUpdate = client.getClusterClient()
                    .updateMember(memberId, urls)
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