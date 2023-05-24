package org.beifengtz.etcd.server.controller;

import io.netty.handler.codec.http.DefaultFullHttpResponse;
import io.netty.handler.codec.http.HttpHeaderNames;
import io.netty.handler.codec.http.HttpResponse;
import io.netty.handler.codec.http.HttpResponseStatus;
import io.netty.handler.codec.http.HttpVersion;
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
    public HttpResponse get() {
        HttpResponse response = new DefaultFullHttpResponse(HttpVersion.HTTP_1_1, HttpResponseStatus.PERMANENT_REDIRECT);
        response.headers().set(HttpHeaderNames.LOCATION, "/index.html");
        return response;
    }
}
