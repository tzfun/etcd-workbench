package org.beifengtz.etcd.server.service;

import io.netty.bootstrap.ServerBootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelFuture;
import io.netty.channel.ChannelHandler;
import io.netty.channel.EventLoopGroup;
import io.netty.util.concurrent.EventExecutorGroup;
import org.beifengtz.etcd.server.handler.HttpHandler;
import org.beifengtz.jvmm.convey.channel.ChannelInitializers;
import org.beifengtz.jvmm.convey.channel.HttpServerChannelInitializer;
import org.beifengtz.jvmm.convey.handler.HandlerProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

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
        long st = System.currentTimeMillis();
        EventLoopGroup boosGroup = ChannelInitializers.newEventLoopGroup(1);
        EventLoopGroup workGroup = ChannelInitializers.newEventLoopGroup(2 * Runtime.getRuntime().availableProcessors() + 1);
        ChannelFuture future = new ServerBootstrap()
                .group(boosGroup, workGroup)
                .channel(ChannelInitializers.serverChannelClass(workGroup))
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

        logger.info("Http server service started on {}, use {} ms", port, System.currentTimeMillis() - st);
        channel = future.channel();
    }
}
