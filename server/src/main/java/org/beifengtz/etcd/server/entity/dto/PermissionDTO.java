package org.beifengtz.etcd.server.entity.dto;

import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.experimental.SuperBuilder;
import org.beifengtz.etcd.server.entity.bo.PermissionBO;

/**
 * description: TODO
 * date: 16:37 2023/7/10
 *
 * @author beifengtz
 */
@Data
@SuperBuilder
@EqualsAndHashCode(callSuper = true)
public class PermissionDTO extends PermissionBO {
    private String sessionId;
    private String role;
}
