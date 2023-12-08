package org.beifengtz.etcd.server.entity;

import lombok.Data;

/**
 * description: TODO
 * date: 16:56 2023/12/8
 *
 * @author beifengtz
 */
@Data
public class TokenPayload {
    private String user;
    private String password;
    private long createTime;

    public static TokenPayload parseFrom(String str) {
        String[] split = str.split(",");
        TokenPayload payload = new TokenPayload();
        payload.setUser(split[0]);
        payload.setPassword(split[1]);
        payload.setCreateTime(Long.parseLong(split[2]));
        return payload;
    }

    @Override
    public String toString() {
        return user + "," + password + "," + createTime;
    }
}
