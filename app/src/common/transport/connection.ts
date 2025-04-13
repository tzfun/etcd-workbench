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

export type HashAlgorithm = "sha256" | "sha512"

export interface SshPrivateKey {
    key: number[],
    passphrase?: string,
    hashAlgorithm?: HashAlgorithm
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
    //  客户端构造完之后会将其设置为 undefined
    keyMonitorList?: KeyMonitorConfig[],
    //  客户端自行构造
    keyCollectionSet?: Set<string>,
    keyMonitorMap?: Record<string, KeyMonitorConfig>,
}

export interface ErrorPayload {
    errType: string,
    errMsg: string
}

export interface KeyMonitorConfig {
    key: string,
    isPrefix: boolean,
    monitorValueChange: boolean,
    monitorCreate: boolean,
    monitorRemove: boolean,
    paused: boolean,
}