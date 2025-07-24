import {writeText} from "@tauri-apps/api/clipboard";
import {emit} from "@tauri-apps/api/event";
import {WebviewWindow} from "@tauri-apps/api/window";
import mitt, {Emitter, EventType, Handler} from "mitt";
import {DialogItem, TipsItem} from "~/common/types.ts";
import {_relativeTimeFormat} from "~/common/utils.ts";
import {KeyValue} from "./transport/kv";
import {KeyMonitorConfig} from "~/common/transport/connection.ts";
import {CustomUpdateManifest} from "~/common/updater.ts";

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
    KEY_MONITOR_MODIFIED_BY_SERVER  ="key_monitor_modified_by_server",
    SET_SETTING_ANCHOR = 'setSettingAnchor',
    SESSION_DISCONNECTED = 'sessionDisconnected',
    UPDATE_AVAILABLE = 'updateAvailable',
    UPDATE_PENDING = 'updatePending',
    UPDATE_DOWNLOADING_PROGRESS = 'updateDownloadingProgress',
    UPDATE_DOWNLOADED = 'updateDownloaded',
    UPDATE_INSTALLED = 'updateInstalled',
    UPDATE_ERRORS = 'updateErrors',
    RENAME_DIR_EVENT = 'renameDirEvent',
    RENAME_DIR_START_EVENT = 'renameDirStartEvent',
    RENAME_DIR_END_EVENT = 'renameDirEndEvent',
    RENAME_DIR_ERR_EVENT = 'renameDirErrEvent',
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
    id?: number,
    eventKey?: string,
}

export interface KeyMonitorModifiedByServerEvent {
    session: number,
    config: KeyMonitorConfig,
}

export interface SessionDisconnectedEvent {
    sessionId: number,
    case: string | Record<string, string>,
}

export interface UpdateDownloadingProgressEvent {
    chunkLength: number,
    contentLength?: number,
}

export type RenameAction = 'Put' | 'Delete'

export interface KVRenameDirEvent {
    key: number[]
    action: RenameAction
    success: boolean
    failedMsg: string
}

export function _useLocalEvents(): Emitter<Record<EventType, any>> {
    return localEvents
}

export function _listenLocal(type: EventName, handler: Handler<any>) {
    localEvents.on(type, handler)
}

export function _unListenLocal(type: EventName, handler: Handler<any>) {
    localEvents.off(type, handler)
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

export function _confirm(title: string, text: string, zIndex?: number): Promise<void> {
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
            ],
            zIndex
        }

        _emitLocal(EventName.DIALOG, dialog)
    })
}

export function _confirmSystem(text: string): Promise<void> {
    return _confirm('Confirm', text)
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

export function _genNewVersionUpdateMessage(manifest: CustomUpdateManifest): string {
    const version = manifest.version
    let message =  `New version <span onclick='_goBrowserPage("https://github.com/tzfun/etcd-workbench/releases/tag/App-${version}")' class="simulate-tag-a text-green font-weight-bold" title="Click to view updated content">${version}</span> released`

    if (manifest.date) {
        const timeDes = _relativeTimeFormat(new Date(manifest.date * 1000))
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
