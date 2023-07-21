package org.beifengtz.etcd.server;

import org.beifengtz.etcd.server.config.Configuration;
import org.beifengtz.etcd.server.service.HttpService;
import org.beifengtz.jvmm.common.util.IOUtil;
import org.beifengtz.jvmm.common.util.StringUtil;
import org.beifengtz.jvmm.common.util.SystemPropertyUtil;
import org.slf4j.LoggerFactory;

import javax.naming.ConfigurationException;
import java.io.File;
import java.io.InputStream;
import java.nio.file.Files;

/**
 * description: TODO
 * date: 14:48 2023/5/23
 *
 * @author beifengtz
 */
public class EtcdServer {

    private static void loadBanner() throws Exception {
        InputStream is = EtcdServer.class.getResourceAsStream("/banner.txt");
        if (is != null) {
            System.out.println(IOUtil.toString(is));
        }
    }

    private static void loadConfiguration() throws Exception {
        String config = null;
        File file = new File(SystemPropertyUtil.get("user.dir"), "app.conf");
        if (file.exists()) {
            config = Files.readString(file.toPath());
        } else {
            InputStream is = EtcdServer.class.getResourceAsStream("/app.conf");
            if (is != null) {
                config = IOUtil.toString(is);
            }
        }
        if (config == null) {
            throw new ConfigurationException("No configuration");
        }
        String[] lines = config.split("\n");
        String part = null;
        for (String line : lines) {
            if (StringUtil.nonEmpty(line) && !line.startsWith("#")) {
                if (line.startsWith("[")) {
                    part = line.substring(1, line.lastIndexOf("]"));
                } else {
                    int i = line.indexOf("=");

                    String key = line.substring(0, i).trim();
                    String value = line.substring(i + 1).trim();
                    if (value.startsWith("\"") && value.endsWith("\"")) {
                        value = value.substring(1, value.length() - 1);
                    }
                    if (StringUtil.isEmpty(value)) {
                        continue;
                    }

                    if ("server".equalsIgnoreCase(part)) {
                        if ("port".equalsIgnoreCase(key)) {
                            int port = Integer.parseInt(value);
                            if (port <= 0) {
                                throw new ConfigurationException("Parameter configuration error, illegal port " + port);
                            }
                            Configuration.INSTANCE.setPort(port);
                        } else if ("username".equalsIgnoreCase(key)) {
                            Configuration.INSTANCE.setUsername(value);
                        } else if ("password".equalsIgnoreCase(key)) {
                            Configuration.INSTANCE.setPassword(value);
                        } else if ("etcdExecuteTimeoutMillis".equals(key)) {
                            Configuration.INSTANCE.setEtcdExecuteTimeoutMillis(Integer.parseInt(value));
                        }
                    } else if ("log".equalsIgnoreCase(part)) {
                        System.setProperty("jvmm.log." + key, value);
                    }
                }
            }
        }
        System.setProperty("jvmm.scanPack", "org.beifengtz.etcd.server");
        LoggerFactory.getLogger(EtcdServer.class).info("Load configuration successfully");
    }

    private static void bootstrap() {
        new HttpService().start(Configuration.INSTANCE.getPort());
    }

    public static void main(String[] args) throws Exception {
        loadBanner();
        loadConfiguration();
        bootstrap();
    }
}
