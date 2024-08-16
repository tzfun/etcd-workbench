import {DialogItem, TipsItem} from "~/common/types.ts";
import {appWindow, WebviewWindow} from "@tauri-apps/api/window";
import {emit} from "@tauri-apps/api/event";

export function _emitLocal(eventName: string, eventPayload: any) {
    appWindow.emit(eventName, eventPayload).then(() => {
    }).catch(e => {
        console.error(e)
    })
}

export function _emitGlobal(eventName: string, eventPayload: any) {
    emit(eventName, eventPayload).then(() => {
    }).catch(e => {
        console.error(e)
    })
}

export function _emitWindow(windowLabel: string, eventName: string, eventPayload: any) {
    let window = WebviewWindow.getByLabel(windowLabel);
    if (!window) {
        window = new WebviewWindow(windowLabel)
    }

    window.emit(eventName, eventPayload).then(() => {
    }).catch(e => {
        console.error(e)
    })
}


export function _loading(state: boolean) {
    _emitLocal('loading', state)
}

export function _confirm(title: string, text: string): Promise<undefined> {
    return new Promise((resolve, reject) => {
        let dialog: DialogItem = {
            value: true,
            content: text,
            title,
            icon: 'mdi-alert-circle-outline',
            iconColor: 'yellow-darken-4',
            buttons: [
                {
                    text: "Cancel",
                    callback: (item: DialogItem) => {
                        item.value = false
                        reject()
                    }
                },
                {
                    text: "Confirm",
                    class: 'text-primary',
                    callback: (item: DialogItem) => {
                        item.value = false
                        resolve(undefined)
                    }
                }
            ]
        }

        _emitLocal('dialog', dialog)
    })

}

export function _confirmSystem(text: string): Promise<undefined> {
    return _confirm('System', text)
}

export function _dialogContent(content: string) {
    let dialog: DialogItem = {
        value: true,
        title: 'Display Content',
        content: content,
        maxWidth: 1200,
        closeBtn: true
    }

    _emitLocal('dialog', dialog)
}

export function _alertError(text: string) {
    let dialog: DialogItem = {
        value: true,
        title: "Error",
        content: text,
        icon: 'mdi-alert-circle-outline',
        iconColor: "red",
        buttons: [
            {
                text: "Close",
                callback: (item: DialogItem) => {
                    item.value = false
                }
            }
        ]
    }

    _emitLocal('dialog', dialog)
}

export function _tipError(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-alert-circle-outline',
        class: 'bg-red-lighten-1'
    }

    _emitLocal('tip', tip)
}

export function _tipWarn(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-alert-circle',
        class: 'bg-orange-darken-1'
    }

    _emitLocal('tip', tip)
}

export function _tipSuccess(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-check',
        class: 'bg-green-lighten-1'
    }

    _emitLocal('tip', tip)
}

export function _tipInfo(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-lightbulb-on-40',
        class: 'bg-secondary'
    }

    _emitLocal('tip', tip)
}