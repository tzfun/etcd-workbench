package org.beifengtz.etcd.server.util;

import io.etcd.jetcd.ByteSequence;

import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * description: TODO
 * date: 10:19 2023/5/26
 *
 * @author beifengtz
 */
public class CommonUtil {
    public static ByteSequence toByteSequence(String str) {
        return ByteSequence.from(str, UTF_8);
    }
}
