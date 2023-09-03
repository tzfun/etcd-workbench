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
    theme: string,
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

export interface SessionConfig {
    namespace: string | ''
    user?: string
    password?: string
    authority?: string
    caType: string
    caCert?: string | null
    clientCertMode?: string | 'none'
    clientCert?: string | null
    clientCertPassword?: string | null
    clientCertKey?: string | null
}

export interface SessionStoreConfig extends SessionConfig {
    name: string
    host: string
    port: number
}
