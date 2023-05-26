package org.beifengtz.etcd.server.entity.vo;

import lombok.Builder;
import lombok.Data;

/**
 * description: TODO
 * date: 11:37 2023/5/26
 *
 * @author beifengtz
 */
@Data
@Builder
public class ResultVO {
    public int code;
    public String msg;
    public Object data;
}
