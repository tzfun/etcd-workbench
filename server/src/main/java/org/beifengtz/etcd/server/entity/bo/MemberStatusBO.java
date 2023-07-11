package org.beifengtz.etcd.server.entity.bo;

import io.etcd.jetcd.maintenance.StatusResponse;
import lombok.Data;
import lombok.extern.slf4j.Slf4j;
import org.beifengtz.jvmm.common.JsonParsable;

/**
 * description: TODO
 * date: 16:44 2023/7/11
 *
 * @author beifengtz
 */
@Data
@Slf4j
public class MemberStatusBO implements JsonParsable {
    private String version;
    private long dbSize;
    private String leader;
    private long raftIndex;
    private long raftTerm;

    public static MemberStatusBO parseFrom(StatusResponse statusResponse) {
        MemberStatusBO status = new MemberStatusBO();
        status.setLeader(Long.toUnsignedString(statusResponse.getLeader()));
        status.setVersion(statusResponse.getVersion());
        status.setDbSize(statusResponse.getDbSize());
        status.setRaftIndex(statusResponse.getRaftIndex());
        status.setRaftTerm(statusResponse.getRaftTerm());
        return status;
    }
}
