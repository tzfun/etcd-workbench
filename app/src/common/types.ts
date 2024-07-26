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
    icon?: string
    buttons?: DialogButton[]
}

export type TipsItem = {
    value: boolean,
    content: string,
    timeout: number,
    icon?: string,
    close?: Function,
    class?: string
}
