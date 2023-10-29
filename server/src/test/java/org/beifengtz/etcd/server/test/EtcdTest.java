package org.beifengtz.etcd.server.test;

import com.jcraft.jsch.Session;
import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.KV;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.options.GetOption;
import io.netty.handler.ssl.ApplicationProtocolConfig;
import io.netty.handler.ssl.ApplicationProtocolNames;
import org.beifengtz.etcd.server.entity.bo.SessionBO;
import org.beifengtz.etcd.server.entity.dto.NewSessionDTO;
import org.beifengtz.etcd.server.entity.dto.SshDTO;
import org.beifengtz.etcd.server.etcd.EtcdConnector;
import org.beifengtz.etcd.server.etcd.EtcdConnectorFactory;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.junit.jupiter.api.Test;

import java.nio.charset.StandardCharsets;
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

        ApplicationProtocolConfig alpn = new ApplicationProtocolConfig(ApplicationProtocolConfig.Protocol.ALPN,
                ApplicationProtocolConfig.SelectorFailureBehavior.NO_ADVERTISE,
                ApplicationProtocolConfig.SelectedListenerFailureBehavior.ACCEPT,
                ApplicationProtocolNames.HTTP_2);

        Client client = Client.builder()
                .target("ip:///127.0.0.1:2379")
                .namespace(ByteSequence.EMPTY)
//                .sslContext(SslContextBuilder
//                        .forClient()
//                        .applicationProtocolConfig(alpn)
//                        .trustManager(InsecureTrustManagerFactory.INSTANCE)
//                        .build())
                .authority("127.0.0.1")
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
    public void testConnector() throws Exception {
        NewSessionDTO config = new NewSessionDTO();
        config.setHost("127.0.0.1");
        config.setPort(2379);
        config.setProtocol("ip");

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
        config.setProtocol("http");
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
}
