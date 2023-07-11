package org.beifengtz.etcd.server.entity.bo;

import lombok.Data;

import java.util.List;

/**
 * description: TODO
 * date: 17:59 2023/7/11
 *
 * @author beifengtz
 */
@Data
public class ClusterBO {
    private double clusterId;
    private double leader;
    private long revision;
    private long raftTerm;
    private List<MemberBO> members = List.of();
}
