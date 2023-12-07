package org.beifengtz.etcd.server.config;

import java.util.HashMap;
import java.util.Map;

/**
 * description: TODO
 * date: 15:41 2023/5/23
 *
 * @author beifengtz
 */
public class Configuration {

    public static final Configuration INSTANCE = new Configuration();

    private int port = 8080;
    private int etcdExecuteTimeoutMillis = 3000;
    private String dataDir = "data";
    private String configEncryptKey = "etcdWorkbench@*?";
    private boolean enableAuth;
    private final Map<String, String> users = new HashMap<>();

    private Configuration() {
    }

    public int getPort() {
        return port;
    }

    public Configuration setPort(int port) {
        this.port = port;
        return this;
    }

    public int getEtcdExecuteTimeoutMillis() {
        return etcdExecuteTimeoutMillis;
    }

    public Configuration setEtcdExecuteTimeoutMillis(int etcdExecuteTimeoutMillis) {
        this.etcdExecuteTimeoutMillis = etcdExecuteTimeoutMillis;
        return this;
    }

    public String getDataDir() {
        return dataDir;
    }

    public Configuration setDataDir(String dataDir) {
        this.dataDir = dataDir;
        return this;
    }

    public String getConfigEncryptKey() {
        return configEncryptKey;
    }

    public Configuration setConfigEncryptKey(String configEncryptKey) {
        this.configEncryptKey = configEncryptKey;
        return this;
    }

    public boolean isEnableAuth() {
        return enableAuth;
    }

    public Configuration setEnableAuth(boolean enableAuth) {
        this.enableAuth = enableAuth;
        return this;
    }

    public Map<String, String> getUsers() {
        return users;
    }

    public void addUser(String user, String password) {
        String previous = users.put(user, password);
        if (previous != null) {
            System.err.println("Warning: exist multi user in [auth] configuration: " + user);
        }
    }
}
