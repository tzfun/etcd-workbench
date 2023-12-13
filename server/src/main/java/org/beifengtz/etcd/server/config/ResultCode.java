package org.beifengtz.etcd.server.config;

import org.beifengtz.etcd.server.entity.vo.ResultVO;

/**
 * description: TODO
 * date: 11:38 2023/5/26
 *
 * @author beifengtz
 */
public enum ResultCode {
    OK(0, "ok"),
    INVALID_KEY(10001, "Invalid key spec"),
    CONNECT_ERROR(10002, "Connect error"),
    ETCD_ERROR(10003, "Etcd server error"),
    PARAM_FORMAT_ERROR(10004, "Param error"),
    NOT_SUPPORTED(10005, "Not supported operation"),
    LOGIN_FAILED(10006, "Please login"),
    ;
    private final int code;
    private final String msg;

    ResultCode(int code, String msg) {
        this.code = code;
        this.msg = msg;
    }

    public int get() {
        return code;
    }

    public ResultVO result() {
        return result(msg);
    }

    public ResultVO result(Object data) {
        return result(msg, data);
    }

    public ResultVO result(String message, Object data) {
        return ResultVO.builder().code(code).msg(message == null ? msg : message).data(data).build();
    }
}
