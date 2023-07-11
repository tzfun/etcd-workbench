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

    public static final String EMPTY_STR = "";

    public static ByteSequence toByteSequence(String str) {
        if (str == null || str.length() == 0) {
            return ByteSequence.EMPTY;
        }
        return ByteSequence.from(str, UTF_8);
    }

    public static String toString(ByteSequence bs) {
        byte[] bytes = bs.getBytes();
        if (bytes.length == 1 && bytes[0] == 0) {
            return EMPTY_STR;
        }
        return bs.toString(UTF_8);
    }
}
