import { writeText } from "@tauri-apps/api/clipboard";
import { emit } from "@tauri-apps/api/event";
import { UpdateManifest } from "@tauri-apps/api/updater";
import { WebviewWindow } from "@tauri-apps/api/window";
import mitt, { Emitter, EventType, Handler } from "mitt";
import { DialogItem, TipsItem } from "~/common/types.ts";
import { _relativeTimeFormat } from "~/common/utils.ts";
import {KeyValue} from "./transport/kv";

const localEvents = mitt();

export enum EventName {
    LOADING = 'loading',
    DIALOG = 'dialog',
    TIP = 'tip',
    CLOSE_TAB = 'closeTab',
    NEW_CONNECTION = 'newConnection',
    SETTING_UPDATE = 'settingUpdate',
    CONNECTION_IMPORTED = 'connectionImported',
    SNAPSHOT_STATE = 'snapshot_state',
    SNAPSHOT_CREATE = 'snapshotCreate',
    CONFIRM_EXIT = 'confirm_exit',
    EDIT_KEY_MONITOR = 'editKeyMonitor',
    KEY_MONITOR_CONFIG_CHANGE = 'keyMonitorChange',
    KEY_WATCH_EVENT = 'key_watch_event',
    KEY_WATCH_ERROR_EVENT = 'key_watch_error_event',
    SET_SETTING_ANCHOR = 'setSettingAnchor',
    SESSION_DISCONNECTED = 'sessionDisconnected',
}

export type KeyWatchEventType = "Remove" | "Create" | "Modify"

export interface KeyWatchEvent {
    session: number,
    key: string,
    eventType: KeyWatchEventType,
    eventTime: number,
    prevKv?: KeyValue,
    curKv?: KeyValue,
    read?: boolean,
    id?: number
}

export interface SessionDisconnectedEvent {
    sessionId: number,
    case: string | Record<string, string>,
}

export function _useLocalEvents(): Emitter<Record<EventType, any>> {
    return localEvents
}

export function _listenLocal(type: EventName, handler: Handler<any>) {
    localEvents.on(type, handler)
}

export function _emitLocal(eventName: EventName, eventPayload?: any) {
    localEvents.emit(eventName, eventPayload)
}

export function _emitGlobal(eventName: EventName, eventPayload?: any) {
    emit(eventName, eventPayload).then(() => {
    }).catch(e => {
        console.error(e)
    })
}

export function _emitWindow(windowLabel: string, eventName: EventName, eventPayload?: any) {
    let window = WebviewWindow.getByLabel(windowLabel);
    if (!window) {
        return
    }

    window.emit(eventName, eventPayload).catch(e => {
        console.error(e)
    })
}


export function _loading(state: boolean, text?: string) {
    _emitLocal(EventName.LOADING, {
        state,
        text
    })
}

export function _confirm(title: string, text: string,): Promise<void> {
    return new Promise<void>((resolve, reject) => {
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
                    variant: "elevated",
                    color: 'primary',
                    callback: (item: DialogItem) => {
                        item.value = false
                        resolve(undefined)
                    }
                }
            ]
        }

        _emitLocal(EventName.DIALOG, dialog)
    })
}

export function _confirmSystem(text: string): Promise<void> {
    return _confirm('System', text)
}

export function _confirmUpdateApp(text: string): Promise<undefined> {
    return new Promise((resolve, reject) => {
        let dialog: DialogItem = {
            value: true,
            content: text,
            title: "Update",
            icon: 'mdi-update',
            iconColor: 'green',
            buttons: [
                {
                    text: "Cancel",
                    callback: (item: DialogItem) => {
                        item.value = false
                        reject()
                    }
                },
                {
                    text: "Install",
                    variant: "elevated",
                    color: 'primary',
                    callback: (item: DialogItem) => {
                        item.value = false
                        resolve(undefined)
                    }
                }
            ]
        }

        _emitLocal(EventName.DIALOG, dialog)
    })
}

export function _dialogContent(content: string) {
    let dialog: DialogItem = {
        value: true,
        title: 'Content',
        content: content,
        maxWidth: 1200,
        closeBtn: true
    }

    _emitLocal(EventName.DIALOG, dialog)
}

export function _alertError(text: string):Promise<void> {
    return _alert(text)
}

export function _alert(text: string, title?: string):Promise<void> {
    return new Promise<void>(resolve => {
        let dialog: DialogItem = {
            value: true,
            title: title ? title : "System",
            content: text,
            icon: 'mdi-alert-circle-outline',
            iconColor: "red",
            buttons: [
                {
                    text: "Close",
                    callback: (item: DialogItem) => {
                        item.value = false
                        resolve()
                    }
                }
            ]
        }

        _emitLocal(EventName.DIALOG, dialog)
    })
}

export function _tipError(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-alert-circle-outline',
        class: 'bg-red-lighten-1'
    }

    _emitLocal(EventName.TIP, tip)
}

export function _tipWarn(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-alert-circle',
        class: 'bg-orange-darken-1'
    }

    _emitLocal(EventName.TIP, tip)
}

export function _tipSuccess(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-check',
        class: 'bg-green-lighten-1'
    }

    _emitLocal(EventName.TIP, tip)
}

export function _tipInfo(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-lightbulb-on-40',
        class: 'bg-secondary'
    }

    _emitLocal(EventName.TIP, tip)
}

export function _genNewVersionUpdateMessage(manifest: UpdateManifest): string {
    const version = manifest.version
    // 使用正则表达式提取日期和时间部分
    const regex = /^(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})/;
    const match = manifest.date.match(regex);

    let timeDes = undefined
    if (match) {
        // 构造新的日期字符串
        const formattedDateString = `${match[1]}T${match[2]}Z`
    
        // 创建 Date 对象
        const dateObject = new Date(formattedDateString)
        timeDes = _relativeTimeFormat(dateObject)
    } else {
        console.debug("The date string format is incorrect", manifest.date)
        
        const dateObject = new Date(parseInt(manifest.date))
        timeDes = _relativeTimeFormat(dateObject)
    }
    let message =  `New version <span onclick='_goBrowserPage("https://github.com/tzfun/etcd-workbench/releases/tag/App-${version}")' class="simulate-tag-a text-green font-weight-bold" title="Click to view updated content">${version}</span> released`

    if(timeDes) {
        message += ` ${timeDes}`
    }
    message += ', install it now?'
    return message
}

export function _copyToClipboard(content: any) {
    if (content) {
        content = content.toString()
    }
    writeText(content).then(() => {
        _tipSuccess("Copied")
    }).catch(e => {
        _tipError("Can not write to clipboard")
        console.error(e)
    })
}
