package org.beifengtz.etcd.server.exception;

/**
 * description: TODO
 * date: 17:14 2023/5/23
 *
 * @author beifengtz
 */
public class EtcdExecuteException extends RuntimeException {
    public EtcdExecuteException() {
    }

    public EtcdExecuteException(String message) {
        super(message);
    }

    public EtcdExecuteException(String message, Throwable cause) {
        super(message, cause);
    }

    public EtcdExecuteException(Throwable cause) {
        super(cause);
    }

    public EtcdExecuteException(String message, Throwable cause, boolean enableSuppression, boolean writableStackTrace) {
        super(message, cause, enableSuppression, writableStackTrace);
    }
}
