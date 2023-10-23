package org.beifengtz.etcd.server.entity.dto;

import lombok.Data;

/**
 * description: TODO
 * date: 10:06 2023/10/23
 *
 * @author beifengtz
 */
@Data
public class SshDTO {
    private String host;
    private int port = 22;
    private String user;
    private String password;
    private String privateKey;
    private String passphrase;
    /**
     * 连接超时时间，单位毫秒
     */
    private int timeout = 30000;
}
