package org.beifengtz.etcd.server.entity.vo;

import lombok.Data;

/**
 * description TODO
 * date 16:04 2024/5/9
 *
 * @author beifengtz
 */
@Data
public class CheckLoginVO {
    private boolean enableAuth;
    private boolean needLogin;
    private String version;
    private String buildHash;
    private boolean enableHeartbeat;
}
