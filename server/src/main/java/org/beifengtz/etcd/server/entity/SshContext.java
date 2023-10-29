package org.beifengtz.etcd.server.entity;

import com.jcraft.jsch.Session;
import lombok.Data;

/**
 * description TODO
 * date 22:06 2023/10/29
 *
 * @author beifengtz
 */
@Data
public class SshContext {
    private Session session;
    /**
     * 源端口
     */
    private int srcPort;
    /**
     * 源 host
     */
    private String srcHost;
    /**
     * 代理后本地端口
     */
    private int proxyLocalPort;
    /**
     * 代理后本地 host
     */
    private String proxyLocalHost;
}
