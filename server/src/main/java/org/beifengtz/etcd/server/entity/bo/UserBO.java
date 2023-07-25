package org.beifengtz.etcd.server.entity.bo;

import lombok.Data;
import lombok.EqualsAndHashCode;
import org.beifengtz.jvmm.common.JsonParsable;

import java.util.List;

/**
 * description: TODO
 * date: 11:09 2023/7/10
 *
 * @author beifengtz
 */
@Data
@EqualsAndHashCode(of = "user")
public class UserBO implements JsonParsable {
    private String user;
    private List<String> roles = List.of();
}
