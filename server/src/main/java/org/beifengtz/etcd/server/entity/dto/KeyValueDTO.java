package org.beifengtz.etcd.server.entity.dto;

import lombok.Data;

/**
 * description TODO
 * date 17:39 2023/7/16
 *
 * @author beifengtz
 */
@Data
public class KeyValueDTO {
    private String sessionId;
    private String key;
    private String value;
    /**
     * 过期时间，单位秒
     */
    private Long ttl;
}
