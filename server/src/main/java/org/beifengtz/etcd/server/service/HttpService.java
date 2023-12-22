package org.beifengtz.etcd.server.service;

import io.netty.bootstrap.ServerBootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelFuture;
import io.netty.channel.ChannelHandler;
import io.netty.channel.EventLoopGroup;
import org.beifengtz.etcd.server.handler.HttpHandler;
import org.beifengtz.jvmm.common.util.IPUtil;
import org.beifengtz.jvmm.convey.channel.ChannelUtil;
import org.beifengtz.jvmm.convey.channel.HttpServerChannelInitializer;
import org.beifengtz.jvmm.convey.handler.HandlerProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.awt.*;
import java.awt.Desktop.Action;
import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;

/**
 * description: TODO
 * date: 15:00 2023/5/23
 *
 * @author beifengtz
 */
public class HttpService {

    public static final String HTTP_SERVER_HANDLER_NAME = "httpHandler";

    protected Channel channel;

    private static final Logger logger = LoggerFactory.getLogger(HttpService.class);

    public void start(int port) {
        HttpHandler.init();
        long st = System.currentTimeMillis();
        EventLoopGroup boosGroup = ChannelUtil.newEventLoopGroup(1);
        EventLoopGroup workGroup = ChannelUtil.newEventLoopGroup(2 * Runtime.getRuntime().availableProcessors() + 1);
        ChannelFuture future = new ServerBootstrap()
                .group(boosGroup, workGroup)
                .channel(ChannelUtil.serverChannelClass(workGroup))
                .childHandler(new HttpServerChannelInitializer(new HandlerProvider() {
                    @Override
                    public ChannelHandler getHandler() {
                        return new HttpHandler();
                    }

                    @Override
                    public String getName() {
                        return HTTP_SERVER_HANDLER_NAME;
                    }

                    @Override
                    public int getIdleSeconds() {
                        return 5;
                    }
                }))
                .bind(port)
                .syncUninterruptibly();

        openBrowser(port);
        logger.info("Http server service started on {} in {} ms", port, System.currentTimeMillis() - st);
        channel = future.channel();
    }

    private void openBrowser(int port) {
        try {
            if (Desktop.isDesktopSupported()) {
                Desktop desktop = Desktop.getDesktop();
                if (desktop.isSupported(Action.BROWSE)) {
                    try {
                        String url = "http://" + IPUtil.getLocalIP() + ":" + port;
                        desktop.browse(new URI(url));
                        logger.info("Opened etcd workbench in browser: {}", url);
                        return;
                    } catch (IOException | URISyntaxException e) {
                        logger.warn("Can not open browser", e);
                    }
                }
            }
        } catch (UnsupportedOperationException e) {
            logger.warn("Can not open browser: {}", e.getMessage());
        }
        logger.info("Please access http://{}:{}", IPUtil.getLocalIP(), port);
    }
}
