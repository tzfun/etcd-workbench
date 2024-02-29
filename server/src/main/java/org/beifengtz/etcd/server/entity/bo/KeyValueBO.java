package org.beifengtz.etcd.server.entity.bo;

import io.etcd.jetcd.KeyValue;
import lombok.Getter;
import lombok.Setter;
import lombok.experimental.SuperBuilder;

import java.nio.charset.StandardCharsets;

/**
 * description: TODO
 * date: 10:28 2023/5/30
 *
 * @author beifengtz
 */
@Getter
@Setter
@SuperBuilder
public class KeyValueBO extends KeyBO {
    private String value;

    public static KeyValueBO parseFrom(KeyValue kv) {
        return KeyValueBO.builder()
                .key(kv.getKey().toString(StandardCharsets.UTF_8))
                .lease(Long.toUnsignedString(kv.getLease()))
                .createRevision(kv.getCreateRevision())
                .modRevision(kv.getModRevision())
                .version(kv.getVersion())
                .value(kv.getValue().toString(StandardCharsets.UTF_8))
                .build();
    }
}
