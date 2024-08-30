export interface ServerInfo {
    enableAuth?: boolean;
    needLogin?: boolean;
    version: string | null;
    buildHash: string | null;
    enableHeartbeat?: boolean;
}

export interface KeyDTO {
    key: string;
    version: number | 0;
    createRevision: number | 0;
    modRevision: number | 0;
    lease?: number | 0;
    ttl?: number;
}

export interface KeyValueDTO extends KeyDTO {
    value: string | undefined;
}

export type EditorConfig = {
    disabled: boolean
    indentWithTab: boolean
    tabSize: number
    autofocus: boolean
    height: string | 'auto'
    language: string
    fontSize: string,
}

export type SessionDTO = {
    sessionId: string
    root: boolean
}

export type TreeNode = {
    path: string,
    type: 'dir' | 'file',
    label: string,
    children?: TreeNode[],
    data?: KeyValueDTO
}

export interface SSHConfig {
    host: string
    port: number
    user: string
    password?: string | null
    privateKey?: string | null
    passphrase?: string | null
    timeout?: number
}

export interface SessionConfig extends Record<string, any> {
    namespace: string | ''
    host: string
    port: number
    user?: string
    password?: string
    authority?: string
    caType: string
    caCert?: string | null
    clientCertMode?: string | 'none'
    clientCert?: string | null
    clientCertPassword?: string | null
    clientCertKey?: string | null
    ssh?: SSHConfig | null
}

export interface SessionStoreConfig extends SessionConfig {
    key?: string,
    name: string,
    enableFunc: {
        auth: false,
        ssl: false,
        ssh: false
    }
}

export interface SessionStoreConfigDict extends Record<string, SessionStoreConfig> {
    [key: string]: SessionStoreConfig
}

export interface MemberStatus {
    version: string
    dbSize: number
    leader: string
    raftIndex: number
    raftTerm: number
}
