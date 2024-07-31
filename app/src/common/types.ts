export type DialogButtonFunc = (item: DialogItem, event: PointerEvent) => void;

export type DialogButton = {
    text: string,
    class?: string,
    icon?: string,
    callback: DialogButtonFunc
}

export type DialogItem = {
    value: boolean,
    title?: string,
    content: string,
    icon?: string,
    buttons?: DialogButton[],
    minWidth?: number,
    maxWidth?: number,
    persistent?: boolean
    scrollable?: boolean
}

export type TipsItem = {
    value: boolean,
    content: string,
    timeout: number,
    icon?: string,
    close?: Function,
    class?: string
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
    passphrase: string
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

export const DefaultConnection:ConnectionForm = {
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
        domain: '',
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
                    content: undefined
                },
                passphrase: ''
            }
        }
    }
}