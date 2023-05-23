package org.beifengtz.etcd.server.etcd;

import io.etcd.jetcd.Client;
import io.etcd.jetcd.resolver.HttpResolverProvider;
import io.etcd.jetcd.resolver.HttpsResolverProvider;
import io.etcd.jetcd.resolver.IPResolverProvider;
import io.grpc.NameResolverRegistry;
import org.beifengtz.jvmm.common.factory.ExecutorFactory;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;

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
        nameResolverRegistry.register(new HttpResolverProvider());
        nameResolverRegistry.register(new HttpsResolverProvider());

        //  每3秒检测一次连接
        ExecutorFactory.getThreadPool().scheduleWithFixedDelay(() -> {
            CONNECTORS.entrySet().removeIf(entry -> entry.getValue().checkRelease());
        }, 3, 3, TimeUnit.SECONDS);

        logger.info("Initialized the connector factory");
    }

    public static EtcdConnector get(String key) {
        return CONNECTORS.get(key);
    }

    public static String newConnector(Client client) {
        EtcdConnector connector = new EtcdConnector(client);
        CONNECTORS.put(connector.getConnKey(), connector);
        logger.debug("Create a new etcd connector {}", connector.getConnKey());
        return connector.getConnKey();
    }

    public static void onClose(String key) {
        CONNECTORS.remove(key);
    }
}
