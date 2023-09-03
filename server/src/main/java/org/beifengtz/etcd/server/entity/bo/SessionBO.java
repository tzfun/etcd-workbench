package org.beifengtz.etcd.server.entity.bo;

import lombok.Data;

/**
 * description TODO
 * date 11:14 2023/9/3
 *
 * @author beifengtz
 */
@Data
public class SessionBO {
    private String sessionId;
    private boolean root;
}
