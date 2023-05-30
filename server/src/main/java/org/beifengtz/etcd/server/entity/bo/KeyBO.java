package org.beifengtz.etcd.server.entity.bo;

import io.etcd.jetcd.KeyValue;
import lombok.Getter;
import lombok.Setter;
import lombok.experimental.SuperBuilder;
import org.beifengtz.jvmm.common.JsonParsable;

import java.nio.charset.StandardCharsets;

/**
 * description: TODO
 * date: 10:24 2023/5/30
 *
 * @author beifengtz
 */
@Getter
@Setter
@SuperBuilder
public class KeyBO implements JsonParsable {

    private String key;
    private long lease;
    private long createRevision;
    private long modRevision;
    private long version;

    @Override
    public String toString() {
        return toJsonStr();
    }

    public static KeyBO parseFrom(KeyValue kv) {
        return KeyBO.builder()
                .key(kv.getKey().toString(StandardCharsets.UTF_8))
                .lease(kv.getLease())
                .createRevision(kv.getCreateRevision())
                .modRevision(kv.getModRevision())
                .version(kv.getVersion())
                .build();
    }
}
