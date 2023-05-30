package org.beifengtz.etcd.server.test;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.Client;
import io.etcd.jetcd.ClientBuilder;
import io.etcd.jetcd.kv.GetResponse;
import io.etcd.jetcd.options.GetOption;
import io.netty.handler.ssl.ApplicationProtocolConfig;
import io.netty.handler.ssl.ApplicationProtocolNames;
import io.netty.handler.ssl.SslContextBuilder;
import io.netty.handler.ssl.util.InsecureTrustManagerFactory;
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
}
