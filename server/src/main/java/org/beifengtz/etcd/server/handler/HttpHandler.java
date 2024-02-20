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
import io.netty.util.AttributeKey;
import io.netty.util.internal.logging.InternalLogger;
import io.netty.util.internal.logging.InternalLoggerFactory;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.config.Mapping;
import org.beifengtz.etcd.server.config.ResultCode;
import org.beifengtz.etcd.server.controller.AuthController;
import org.beifengtz.etcd.server.exceptions.EtcdSessionLostException;
import org.beifengtz.etcd.server.util.CommonUtil;
import org.beifengtz.jvmm.common.util.IOUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.convey.channel.ChannelUtil;
import org.beifengtz.jvmm.convey.handler.HttpChannelHandler;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.security.NoSuchAlgorithmException;
import java.security.spec.InvalidKeySpecException;
import java.util.Base64;
import java.util.List;
import java.util.Map.Entry;
import java.util.Objects;
import java.util.concurrent.TimeoutException;

/**
 * description: TODO
 * date: 14:59 2023/5/23
 *
 * @author beifengtz
 */
public class HttpHandler extends HttpChannelHandler {

    private static final InternalLogger logger = InternalLoggerFactory.getInstance(HttpHandler.class);

    public static final AttributeKey<String> ATTR_USER = AttributeKey.valueOf("user");

    static {
        globalHeaders.remove(HttpHeaderNames.ACCESS_CONTROL_ALLOW_HEADERS);
        globalHeaders.put(HttpHeaderNames.ACCESS_CONTROL_ALLOW_HEADERS, "content-type,authorization");
        globalHeaders.put(HttpHeaderNames.ACCESS_CONTROL_ALLOW_ORIGIN, "*");
        logger.info("Initialization of http handler is completed");
    }

    @Override
    public InternalLogger logger() {
        return logger;
    }

    @Override
    protected boolean handleBefore(ChannelHandlerContext ctx, String uri, FullHttpRequest msg) {
        if (uri != null && uri.startsWith(Mapping.PRIVATE_API_PREFIX)) {
            if (Configuration.INSTANCE.isEnableAuth()) {
                String authStr = msg.headers().get("Authorization");
                if (StringUtil.isEmpty(authStr)) {
                    response401(ctx);
                    return false;
                }
                String[] split = authStr.split("\\s");
                String authType = split[0];
                String authContent = split[1];
                if (authType.equals("Basic")) {
                    try {
                        String[] up = new String(Base64.getDecoder().decode(authContent), StandardCharsets.UTF_8).split(":");
                        String user = up[0];
                        String password = up[1];
                        String userPassword = Configuration.INSTANCE.getUsers().get(user);
                        if (!Objects.equals(userPassword, password)) {
                            response401(ctx);
                            return false;
                        }
                        ctx.channel().attr(ATTR_USER).set(user);
                    } catch (Exception e) {
                        response401(ctx);
                        return false;
                    }
                } else if (authType.equals("Token")) {
                    try {
                        String user = AuthController.verifyToken(authContent);
                        ctx.channel().attr(ATTR_USER).set(user);
                    } catch (Exception e) {
                        response401(ctx);
                        return false;
                    }
                } else {
                    response401(ctx);
                    return false;
                }
            } else {
                ctx.channel().attr(ATTR_USER).set(Configuration.DEFAULT_SYSTEM_USER);
            }
        }

        if (uri == null || !uri.endsWith("heart_beat")) {
            String sendIp = ChannelUtil.getIpByCtx(ctx);
            String srcIp = CommonUtil.getHttpRealIp(msg);
            logger.info("Http request {} {} ({} / {})", msg.method(), uri, sendIp, srcIp);
        }

        return true;
    }

    @Override
    protected void handleFinally(ChannelHandlerContext ctx, FullHttpRequest msg) {

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

    @Override
    protected void handleException(ChannelHandlerContext ctx, FullHttpRequest req, Throwable e) {
        if (e instanceof InvalidKeySpecException || e instanceof NoSuchAlgorithmException) {
            logger.error(e.getMessage(), e);
            response(ctx, HttpResponseStatus.OK, ResultCode.INVALID_KEY.result("Invalid key spec: " + (e.getMessage() == null ? "" : e.getMessage()), false).toString());
        } else if (e instanceof TimeoutException) {
            logger.debug(e.getMessage(), e);
            response(ctx, HttpResponseStatus.OK, ResultCode.CONNECT_ERROR.result("Connect timeout " + e.getMessage(), null).toString());
        } else if (e instanceof IllegalArgumentException) {
            logger.debug(e.getMessage(), e);
            response(ctx, HttpResponseStatus.BAD_REQUEST);
        } else if (e instanceof EtcdSessionLostException) {
            response(ctx, HttpResponseStatus.OK, ResultCode.ETCD_SESSION_LOST.result(e.getMessage(), null).toString());
        }else {
            super.handleException(ctx, req, e);
        }
    }
}
