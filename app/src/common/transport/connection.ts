export interface ConnectionUser {
    username: string,
    password: string
}

export interface TlsIdentity {
    cert: number[],
    key: number[]
}

export interface ConnectionTls {
    domain?: string,
    cert: number[][],
    identity?: TlsIdentity
}

export interface SshPrivateKey {
    key: number[],
    passphrase?: string
}

export interface SshIdentity {
    password?: string,
    key?: SshPrivateKey
}

export interface ConnectionSsh {
    host: string,
    port: number,
    user: string,
    identity?: SshIdentity
}

export interface Connection {
    host: string,
    port: number,
    namespace?: string,
    user?: ConnectionUser,
    tls?: ConnectionTls,
    ssh?: ConnectionSsh
}

export interface ConnectionInfo {
    name: string,
    connection: Connection,
    keyCollection: string[],
    keyMonitorList: KeyMonitorConfig[],
    default?: boolean
}

export const DEFAULT_CONNECTION: ConnectionInfo = {
    name: '',
    connection: {
        host: '',
        port: 2379
    },
    default: true,
    keyCollection: [],
    keyMonitorList: []
}

export interface SessionData {
    id: number,
    user?: string,
    root: boolean,
    connectionSaved: boolean,
    namespace?: string,
    keyCollection?: string[],
    keyMonitorList?: KeyMonitorConfig[],
    //  客户端自行构造
    keyCollectionSet?: Set<string>
    keyMonitor?: KeyMonitor
}

export interface ErrorPayload {
    errType: string,
    errMsg: string
}

export interface KeyMonitorConfig {
    key: string,
    monitorLeaseChange: boolean,
    monitorValueChange: boolean,
    monitorCreate: boolean,
    monitorRemove: boolean,
}

export interface KeyMonitor {
    list: string[],
    map: Record<string, KeyMonitorConfig>
}