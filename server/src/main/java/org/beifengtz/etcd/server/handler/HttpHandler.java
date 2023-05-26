package org.beifengtz.etcd.server.handler;

import io.netty.buffer.Unpooled;
import io.netty.channel.ChannelFutureListener;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.http.DefaultFullHttpResponse;
import io.netty.handler.codec.http.FullHttpRequest;
import io.netty.handler.codec.http.HttpHeaderNames;
import io.netty.handler.codec.http.HttpResponse;
import io.netty.handler.codec.http.HttpResponseStatus;
import io.netty.handler.codec.http.HttpVersion;
import io.netty.util.AsciiString;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.jvmm.common.util.IOUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.convey.handler.HttpChannelHandler;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Objects;

/**
 * description: TODO
 * date: 14:59 2023/5/23
 *
 * @author beifengtz
 */
public class HttpHandler extends HttpChannelHandler {

    static {
        globalHeaders.put(HttpHeaderNames.ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    }

    @Override
    public Logger logger() {
        return LoggerFactory.getLogger(HttpHandler.class);
    }

    @Override
    protected boolean handleBefore(ChannelHandlerContext ctx, String uri, FullHttpRequest msg) {
//        if (Configuration.INSTANCE.isEnableAuth()) {
//            String authStr = msg.headers().get("Authorization");
//            if (StringUtil.isEmpty(authStr) || !authStr.startsWith("Basic")) {
//                response401(ctx);
//                return false;
//            }
//            try {
//                String[] up = new String(Base64.getDecoder().decode(authStr.split("\\s")[1]), StandardCharsets.UTF_8).split(":");
//                if (!Objects.equals(Configuration.INSTANCE.getUsername(), up[0]) || !Objects.equals(Configuration.INSTANCE.getPassword(), up[1])) {
//                    response401(ctx);
//                    return false;
//                }
//            } catch (Exception e) {
//                response401(ctx);
//                return false;
//            }
//        }
        return true;
    }

    @Override
    protected boolean handleUnmapping(ChannelHandlerContext ctx, String path, FullHttpRequest msg) {
        InputStream is = getClass().getResourceAsStream("/static" + path);
        if (is == null) {
            return false;
        }
        try {
            byte[] data = IOUtil.toByteArray(is);
            HttpResponse resp = new DefaultFullHttpResponse(HttpVersion.HTTP_1_1, HttpResponseStatus.OK, Unpooled.copiedBuffer(data));
            resp.headers().set(HttpHeaderNames.CONTENT_LENGTH, data.length);
            resp.headers().set(HttpHeaderNames.CONTENT_ENCODING, "UTF-8");

            for (Entry<AsciiString, List<String>> en : globalHeaders.entrySet()) {
                resp.headers().set(en.getKey(), en.getValue());
            }

            if (path.endsWith(".html")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "text/html;charset=utf-8");
            } else if (path.endsWith(".css")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "text/css;charset=utf-8");
            } else if (path.endsWith(".js")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "application/javascript");
            } else if (path.endsWith("woff")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "application/font-woff");
            } else if (path.endsWith("ttf")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "application/font-ttf");
            } else if (path.endsWith(".png")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "image/png");
            } else if (path.endsWith(".jpg") || path.endsWith(".jpeg")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "image/jpeg");
            } else if (path.endsWith(".svg")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "image/svg+xml");
            } else if (path.endsWith(".awebp")) {
                resp.headers().set(HttpHeaderNames.CONTENT_TYPE, "image/webp");
            }

            ctx.writeAndFlush(resp).addListener(ChannelFutureListener.CLOSE);
        } catch (IOException e) {
            response500(ctx, e.getMessage());
        }
        return true;
    }
}
