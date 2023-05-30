package org.beifengtz.etcd.server.entity.vo;

import lombok.Builder;
import lombok.Data;
import org.beifengtz.jvmm.common.JsonParsable;

/**
 * description: TODO
 * date: 11:37 2023/5/26
 *
 * @author beifengtz
 */
@Data
@Builder
public class ResultVO implements JsonParsable {
    public int code;
    public String msg;
    public Object data;

    @Override
    public String toString() {
        return toJsonStr();
    }
}
