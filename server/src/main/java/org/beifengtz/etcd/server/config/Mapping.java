package org.beifengtz.etcd.server.config;

/**
 * description: TODO
 * date: 15:01 2023/12/7
 *
 * @author beifengtz
 */
public interface Mapping {
    String API_PREFIX = "/beifengtz";
    String PUBLIC_API_PREFIX = API_PREFIX + "/pub";
    String PRIVATE_API_PREFIX = API_PREFIX + "/pri";
}
