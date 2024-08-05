export interface ConnectionUser {
    username: string,
    password: string
}

export interface TlsIdentity {
    cert: Uint8Array,
    key: Uint8Array
}

export interface ConnectionTls {
    domain?: string,
    cert: Uint8Array[],
    identity?: TlsIdentity
}

export interface SshPrivateKey {
    key: Uint8Array,
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
    default: boolean
}

export const DEFAULT_CONNECTION: ConnectionInfo = {
    name: '',
    connection: {
        host: '',
        port: 2379
    },
    default: true
}

export interface SessionData {
    id: number,
    user?: string,
    root: boolean
}