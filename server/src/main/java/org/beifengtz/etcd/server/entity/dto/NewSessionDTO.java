package org.beifengtz.etcd.server.entity.dto;

import lombok.Data;

/**
 * description: TODO
 * date: 18:32 2023/5/25
 *
 * @author beifengtz
 */
@Data
public class NewSessionDTO {
    private String namespace;
    private String protocol;
    private String host;
    private int port;
    private String user;
    private String password;
    private String authority;
    /**
     * CA证书的类型
     * none | custom | public
     */
    private String caType = "none";
    /**
     * ca证书内容
     */
    private String caCert;
    /**
     * 客户端验证模式
     * none | password | key
     */
    private String clientCertMode = "none";
    /**
     * 客户端验证的cert证书
     */
    private String clientCert;
    /**
     * 客户端验证的cert证书密码，clientCertMode为password生效
     */
    private String clientCertPassword;
    /**
     * 客户端验证的cert证书密钥，clientCertMode为key生效
     */
    private String clientCertKey;
    private SshDTO ssh;
}
