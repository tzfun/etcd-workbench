package org.beifengtz.etcd.server.etcd;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.KeyValue;
import io.etcd.jetcd.common.exception.ClosedClientException;
import io.etcd.jetcd.kv.DeleteResponse;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.kv.PutResponse;
import io.etcd.jetcd.kv.TxnResponse;
import io.etcd.jetcd.op.Cmp;
import io.etcd.jetcd.op.CmpTarget;
import io.etcd.jetcd.op.Op;
import io.etcd.jetcd.options.DeleteOption;
import io.etcd.jetcd.options.GetOption;
import io.etcd.jetcd.options.PutOption;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.exception.EtcdExecuteException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

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

    private final String key;
    private final Client client;
    private long activeTime;

    public EtcdConnector(Client client) {
        this.client = client;
        this.key = UUID.randomUUID().toString().replaceAll("-", "").toLowerCase(Locale.ROOT);
        this.activeTime = System.currentTimeMillis();
        client.close();
    }

    public String getKey() {
        return key;
    }

    private void onActive() {
        activeTime = System.currentTimeMillis();
    }

    boolean checkRelease() {
        //  3分钟无响应，释放连接
        if (System.currentTimeMillis() - activeTime > TimeUnit.MINUTES.toMillis(3)) {
            client.close();
            logger.debug("Connector closed by factory monitor. {}", key);
            return true;
        }
        return false;
    }

    public void close() {
        client.close();
        EtcdConnectorFactory.onClose(key);
        logger.debug("Connector closed by invoke. {}", key);
    }

    private void onExecuteError(Throwable e) throws EtcdExecuteException {
        logger.error("Etcd client execute failed. " + e.getClass().getName() + ": " + e.getMessage(), e);
        if (e instanceof ClosedClientException) {
            close();
        }
        throw new EtcdExecuteException(e);
    }

    private ByteSequence toByte(String str) {
        return ByteSequence.from(str, UTF_8);
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
                    .get(toByte(key), option)
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
            GetResponse resp = client.getKVClient().get(toByte(key), GetOption.newBuilder().isPrefix(isPrefix).build())
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
     * 设置键值对
     *
     * @param key   键
     * @param value 值
     * @return 返回设置值之前的值
     */
    public String kvPut(String key, String value) {
        onActive();
        try {
            PutResponse resp = client.getKVClient().put(toByte(key), toByte(value)).get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
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
        ByteSequence bsKey = toByte(key);
        ByteSequence bsExpectValue = toByte(expectValue);
        ByteSequence bsUpdateValue = toByte(updateValue);

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
                    .delete(toByte(key), DeleteOption.newBuilder().isPrefix(isPrefix).build())
                    .get(Configuration.INSTANCE.getEtcdExecuteTimeoutMillis(), TimeUnit.MILLISECONDS);
            return resp.getDeleted();
        } catch (InterruptedException | ExecutionException | TimeoutException e) {
            onExecuteError(e);
        }
        return 0;
    }
}
