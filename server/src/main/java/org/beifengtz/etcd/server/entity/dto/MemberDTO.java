package org.beifengtz.etcd.server.entity.dto;

import lombok.Data;

import java.util.List;

/**
 * description: TODO
 * date: 16:11 2023/7/11
 *
 * @author beifengtz
 */
@Data
public class MemberDTO {
    private String sessionId;
    private String memberId;
    private List<String> urlList;
}
