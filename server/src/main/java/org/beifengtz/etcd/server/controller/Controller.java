package org.beifengtz.etcd.server.controller;

import org.beifengtz.jvmm.convey.annotation.HttpController;
import org.beifengtz.jvmm.convey.annotation.HttpRequest;

/**
 * description: TODO
 * date: 14:57 2023/5/23
 *
 * @author beifengtz
 */
@HttpController
public class Controller {

    @HttpRequest("/")
    public String get() {
        return "hello world";
    }
}
