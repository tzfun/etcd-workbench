import {ErrorPayload, HashAlgorithm, SessionData} from "~/common/transport/connection.ts";

export type EditorHighlightLanguage = EditorSupportedHighlightLanguage | EditorNotSupportedHighlightLanguage

export type EditorSupportedHighlightLanguage =
    'text'
    | 'blob'
    | 'json'
    | 'yaml'
    | 'xml'
    | 'sql'
    | 'properties'
    | 'shell'
    | 'dockerfile'
    | 'nginx'
    | 'kubernetes'

export type EditorNotSupportedHighlightLanguage = 'markdown' | 'js' | 'ts'

export type DialogButtonFunc = (item: DialogItem, event: PointerEvent) => void;

export type DialogButton = {
    text: string,
    class?: string,
    icon?: string,
    variant?: "flat" | "text" | "elevated" | "tonal" | "outlined" | "plain",
    color?: string,
    callback: DialogButtonFunc
}

export type DialogItem = {
    value: boolean,
    title?: string,
    content: string,
    icon?: string,
    iconColor?: string,
    buttons?: DialogButton[],
    minWidth?: number,
    maxWidth?: number,
    persistent?: boolean
    scrollable?: boolean,
    closeBtn?: boolean,
    zIndex?: number,
    preview?: boolean,
}

export type TipsItem = {
    value: boolean,
    content: string,
    timeout: number,
    icon?: string,
    close?: Function,
    class?: string,
    id?: number
}

export type FileForm = {
    file?: File,
    content?: string
}

export type ConnectionForm = {
    name: string,
    host: string,
    port: string,
    namespace: string,
    user: ConnectionUserForm,
    tls: ConnectionTlsForm,
    ssh: ConnectionSshForm
}

export type ConnectionUserForm = {
    enable: boolean,
    username: string,
    password: string
}

export type ConnectionTlsIdentity = {
    enable: boolean,
    cert: FileForm,
    key: FileForm
}

export type ConnectionTlsForm = {
    enable: boolean,
    domain: string,
    cert: FileForm,
    identity: ConnectionTlsIdentity
}

export type ConnectionSshKey = {
    key: FileForm,
    passphrase: string,
    hashAlgorithm?: HashAlgorithm | ""
}

export type ConnectionSshIdentity = {
    model: 'password' | 'key' | 'none',
    password: string,
    key: ConnectionSshKey
}

export type ConnectionSshForm = {
    enable: boolean,
    host: string,
    port: string,
    user: string,
    identity: ConnectionSshIdentity,
}

export const DefaultConnection: ConnectionForm = {
    name: '',
    host: '',
    port: '2379',
    namespace: '',
    user: {
        enable: false,
        username: '',
        password: ''
    },
    tls: {
        enable: false,
        domain: '127.0.0.1',
        cert: {
            file: undefined,
            content: undefined
        },
        identity: {
            enable: false,
            cert: {
                file: undefined,
                content: undefined
            },
            key: {
                file: undefined,
                content: undefined
            }
        }
    },
    ssh: {
        enable: false,
        host: '',
        port: '22',
        user: '',
        identity: {
            model: 'none',
            password: '',
            key: {
                key: {
                    file: undefined,
                    content: undefined,
                },
                passphrase: '',
                hashAlgorithm: ''
            }
        }
    }
}

export type EditorConfig = {
    disabled: boolean
    indentWithTab: boolean
    tabSize: number
    autofocus: boolean
    height: string | 'auto'
    language: EditorHighlightLanguage
    fontSize: string,
}

export type LogicErrorInfo = {
    e: ErrorPayload | string,
    prefix?: string,
    session?: SessionData
}

export type AppTheme = 'light' | 'dark' | 'auto'

export type UpdateInfo = {
    state: 'none' | 'available' | 'pending' | 'downloading' | 'downloaded' | 'installed' | 'error'
    chunkLength: number
    contentLength: number
    error: string
}