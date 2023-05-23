package org.beifengtz.etcd.server.handler;

import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.http.FullHttpRequest;
import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.convey.handler.HttpChannelHandler;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.Objects;

/**
 * description: TODO
 * date: 14:59 2023/5/23
 *
 * @author beifengtz
 */
public class HttpHandler extends HttpChannelHandler {
    @Override
    public Logger logger() {
        return LoggerFactory.getLogger(HttpHandler.class);
    }

    @Override
    protected boolean handleBefore(ChannelHandlerContext ctx, String uri, FullHttpRequest msg) {
        if (Configuration.INSTANCE.isEnableAuth()) {
            String authStr = msg.headers().get("Authorization");
            if (StringUtil.isEmpty(authStr) || !authStr.startsWith("Basic")) {
                response401(ctx);
                return false;
            }
            try {
                String[] up = new String(Base64.getDecoder().decode(authStr.split("\\s")[1]), StandardCharsets.UTF_8).split(":");
                if (!Objects.equals(Configuration.INSTANCE.getUsername(), up[0]) || !Objects.equals(Configuration.INSTANCE.getPassword(), up[1])) {
                    response401(ctx);
                    return false;
                }
            } catch (Exception e) {
                response401(ctx);
                return false;
            }
        }
        return true;
    }
}
