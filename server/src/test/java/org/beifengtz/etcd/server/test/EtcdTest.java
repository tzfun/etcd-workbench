package org.beifengtz.etcd.server.test;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.ClientBuilder;
import io.etcd.jetcd.KV;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.options.GetOption;
import io.netty.handler.ssl.ApplicationProtocolConfig;
import io.netty.handler.ssl.ApplicationProtocolNames;
import io.netty.handler.ssl.SslContextBuilder;
import io.netty.handler.ssl.SslProvider;
import org.beifengtz.etcd.server.entity.bo.KeyValueBO;
import org.beifengtz.etcd.server.entity.dto.NewSessionDTO;
import org.beifengtz.etcd.server.entity.dto.SshDTO;
import org.beifengtz.etcd.server.etcd.EtcdConnector;
import org.beifengtz.etcd.server.etcd.EtcdConnectorFactory;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.junit.jupiter.api.Test;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.concurrent.TimeUnit;

/**
 * description: TODO
 * date: 18:26 2023/5/29
 *
 * @author beifengtz
 */
public class EtcdTest {
    @Test
    public void testConnect() throws Exception {
        Client client = Client.builder()
                .target("ip:///127.0.0.1:2379")
                .namespace(ByteSequence.EMPTY)
                .build();
        GetResponse resp = client.getKVClient().get(CommonUtil.toByteSequence(" "), GetOption.newBuilder()
                        .withKeysOnly(true)
                        .isPrefix(true)
                        .withRange(CommonUtil.toByteSequence("\0"))
                        .build())
                .get(5, TimeUnit.SECONDS);
        resp.getKvs().forEach(kv -> {
            System.out.println(kv.getKey().toString(StandardCharsets.UTF_8) + " = " + kv.getValue().toString(StandardCharsets.UTF_8) + ", lease = " +
                    kv.getLease() + ", CreateRevision = " + kv.getCreateRevision() + ", ModRevision = " + kv.getModRevision() + ", version = " + kv.getVersion());
        });
    }

    @Test
    public void testSslConnect() throws Exception {
        File caFile = new File("cert-key/ca.crt");
        File certFile = new File("cert-key/client.crt");
        File certKeyFile = new File("cert-key/client.key");
        ApplicationProtocolConfig alpn = new ApplicationProtocolConfig(ApplicationProtocolConfig.Protocol.ALPN,
                ApplicationProtocolConfig.SelectorFailureBehavior.NO_ADVERTISE,
                ApplicationProtocolConfig.SelectedListenerFailureBehavior.ACCEPT,
                ApplicationProtocolNames.HTTP_2);

        SslContextBuilder sslBuilder = SslContextBuilder
                .forClient()
                .applicationProtocolConfig(alpn)
                .sslProvider(SslProvider.JDK)
                .trustManager(caFile)
                .keyManager(certFile, certKeyFile);
        ClientBuilder clientBuilder = Client.builder()
                .target("ip:///127.0.0.1:2379")
                .namespace(ByteSequence.EMPTY)
                .sslContext(sslBuilder.build());
        System.out.println(StringUtil.getGson().toJson(clientBuilder));
        Client client = clientBuilder.build();
        GetResponse resp = client.getKVClient().get(CommonUtil.toByteSequence(" "), GetOption.newBuilder()
                        .withKeysOnly(true)
                        .isPrefix(true)
                        .withRange(CommonUtil.toByteSequence("\0"))
                        .build())
                .get(5, TimeUnit.SECONDS);
        System.out.println(resp);
        resp.getKvs().forEach(kv -> {
            System.out.println(kv.getKey().toString(StandardCharsets.UTF_8) + " = " + kv.getValue().toString(StandardCharsets.UTF_8) + ", lease = " +
                    kv.getLease() + ", CreateRevision = " + kv.getCreateRevision() + ", ModRevision = " + kv.getModRevision() + ", version = " + kv.getVersion());
        });
    }

    @Test
    public void testConnector() throws Exception {
        NewSessionDTO config = new NewSessionDTO();
        config.setHost("127.0.0.1");
        config.setPort(2379);

        EtcdConnector connector = EtcdConnectorFactory.newConnector(config);
        GetOption option = GetOption.newBuilder()
                .withKeysOnly(true)
                .isPrefix(true)
                .withRange(ByteSequence.EMPTY)
                .build();
        System.out.println(connector.kvGet("/", option));
    }

    @Test
    public void testEtcd() throws Exception {
        int loop = 10;

        for (int i = 0; i < loop; i++) {
            Client client = Client.builder()
                    .target("ip:///127.0.0.1:2379")
                    .namespace(ByteSequence.EMPTY)
                    .build();
            KV kvClient = client.getKVClient();

            long count = kvClient.get(CommonUtil.toByteSequence(" ")).get(3, TimeUnit.SECONDS).getCount();
            System.out.println(count);
        }
        Thread.sleep(100000);
    }

    @Test
    public void testSshTunnelConnect() throws Exception {
        NewSessionDTO config = new NewSessionDTO();
        config.setHost("127.0.0.1");
        config.setPort(2379);
        config.setNamespace("xxxx");
        config.setUser("xxxx");
        config.setPassword("xxx");

        SshDTO ssh = new SshDTO();
        ssh.setHost("xxx");
        ssh.setUser("xxx");
        ssh.setPrivateKey("xxx");
        ssh.setPassphrase("xxx");
        ssh.setPort(22);

        config.setSsh(ssh);

        EtcdConnector connector = EtcdConnectorFactory.newConnector(config);
        GetOption option = GetOption.newBuilder()
                .withKeysOnly(true)
                .isPrefix(true)
                .withRange(ByteSequence.EMPTY)
                .build();
        System.out.println(connector.kvGet("/", option).get());
    }

    @Test
    public void testPagination() throws Exception {
        NewSessionDTO config = new NewSessionDTO();
        config.setHost("127.0.0.1");
        config.setPort(2379);

        EtcdConnector connector = EtcdConnectorFactory.newConnector(config);
        String cursorKey = null;
        int page = 1;
        final int LIMIT = 2;
        while (true) {
            List<KeyValueBO> kvList = connector.kvGetAllKeysPaging(cursorKey, LIMIT).get();
            System.out.println(page + " ==> " + kvList);
            if (kvList.size() < LIMIT) {
                break;
            }
            cursorKey = kvList.get(kvList.size() - 1).getKey() + "\0";
            page++;
        }

    }
}
