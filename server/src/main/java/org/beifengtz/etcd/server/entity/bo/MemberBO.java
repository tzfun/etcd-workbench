package org.beifengtz.etcd.server.entity.bo;

import io.etcd.jetcd.cluster.Member;
import lombok.Data;
import lombok.extern.slf4j.Slf4j;
import org.beifengtz.jvmm.common.JsonParsable;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;

/**
 * description: TODO
 * date: 15:52 2023/7/11
 *
 * @author beifengtz
 */
@Data
@Slf4j
public class MemberBO implements JsonParsable {
    private String id;
    private String name;
    private List<String> peerUri = List.of();
    private List<String> clientUri = List.of();
    private MemberStatusBO status;

    public static MemberBO parseFrom(Member member) {
        MemberBO memberBO = new MemberBO();
        memberBO.setId(Long.toUnsignedString(member.getId()));
        memberBO.setName(member.getName());

        List<URI> peerURIs = member.getPeerURIs();
        if (peerURIs != null && peerURIs.size() > 0) {
            List<String> list = new ArrayList<>(peerURIs.size());
            for (URI uri : peerURIs) {
                list.add(uri.toString());
            }
            memberBO.setPeerUri(list);
        }

        List<URI> clientURIs = member.getClientURIs();
        if (clientURIs != null && clientURIs.size() > 0) {
            List<String> list = new ArrayList<>(clientURIs.size());
            for (URI uri : clientURIs) {
                list.add(uri.toString());
            }
            memberBO.setClientUri(list);
        }
        return memberBO;
    }
}
