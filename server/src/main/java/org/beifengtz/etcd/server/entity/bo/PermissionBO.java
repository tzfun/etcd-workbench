package org.beifengtz.etcd.server.entity.bo;

import io.etcd.jetcd.ByteSequence;
import io.etcd.jetcd.auth.Permission.Type;
import lombok.Data;
import lombok.experimental.SuperBuilder;
import org.beifengtz.jvmm.common.JsonParsable;

import java.nio.charset.StandardCharsets;

/**
 * description: TODO
 * date: 15:08 2023/7/10
 *
 * @author beifengtz
 */
@Data
@SuperBuilder
public class PermissionBO implements JsonParsable {
    private Type type;
    private String key;
    private boolean prefix;
    private boolean allKeys;

    public ByteSequence parseRangeEnd() {
        if (allKeys) {
            return ByteSequence.EMPTY;
        } else if (prefix) {
            byte[] bytes = key.getBytes(StandardCharsets.UTF_8);
            if (bytes[bytes.length - 1] == Byte.MAX_VALUE) {
                byte[] newBytes = new byte[bytes.length + 1];
                System.arraycopy(bytes, 0, newBytes, 0, bytes.length);
                newBytes[newBytes.length - 1] = 1;
                return ByteSequence.from(newBytes);
            } else {
                bytes[bytes.length - 1]++;
                return ByteSequence.from(bytes);
            }
        } else {
            return ByteSequence.from("", StandardCharsets.UTF_8);
        }
    }
}
