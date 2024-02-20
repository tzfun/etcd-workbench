package org.beifengtz.etcd.server.exceptions;

/**
 * description TODO
 * date 14:23 2024/2/20
 *
 * @author beifengtz
 */
public class EtcdSessionLostException extends RuntimeException {
    public EtcdSessionLostException() {
    }

    public EtcdSessionLostException(String message) {
        super(message);
    }

    public EtcdSessionLostException(String message, Throwable cause) {
        super(message, cause);
    }

    public EtcdSessionLostException(Throwable cause) {
        super(cause);
    }

    public EtcdSessionLostException(String message, Throwable cause, boolean enableSuppression, boolean writableStackTrace) {
        super(message, cause, enableSuppression, writableStackTrace);
    }
}
