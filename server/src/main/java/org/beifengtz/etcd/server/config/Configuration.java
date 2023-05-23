package org.beifengtz.etcd.server.config;

/**
 * description: TODO
 * date: 15:41 2023/5/23
 *
 * @author beifengtz
 */
public class Configuration {

    public static final Configuration INSTANCE = new Configuration();

    private int port = 8080;
    private String username;
    private String password;

    private Configuration() {
    }

    public int getPort() {
        return port;
    }

    public Configuration setPort(int port) {
        this.port = port;
        return this;
    }

    public String getUsername() {
        return username;
    }

    public Configuration setUsername(String username) {
        this.username = username;
        return this;
    }

    public String getPassword() {
        return password;
    }

    public Configuration setPassword(String password) {
        this.password = password;
        return this;
    }

    public boolean isEnableAuth() {
        return username != null;
    }
}
