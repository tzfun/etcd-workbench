import { DialogItem, TipsItem } from "~/common/types.ts";
import { WebviewWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import mitt, { Emitter, EventType, Handler } from "mitt";
import { checkUpdate, installUpdate, UpdateManifest, UpdateResult } from "@tauri-apps/api/updater";
import { _useSettings } from "~/common/store.ts";
import { relaunch } from "@tauri-apps/api/process";
import { writeText } from "@tauri-apps/api/clipboard";
import { _goBrowserPage, _relativeTimeFormat } from "~/common/utils.ts";
import { FormattedValue } from "./transport/kv";

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
    KEY_MONITOR_EVENT = 'key_monitor',
    SET_SETTING_ANCHOR = 'setSettingAnchor'
}

export type KeyMonitorEventType = "Remove" | "Create" | "LeaseChange" | "ValueChange"

export interface KeyMonitorEvent {
    session: number,
    key: string,
    eventType: KeyMonitorEventType,
    eventTime: number,
    previous: null | any,
    current: null | any,
    previousFormatted?: FormattedValue
    currentFormatted?: FormattedValue
    read?: boolean,
    id?: number
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

export function _confirm(title: string, text: string,): Promise<undefined> {
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

export function _confirmSystem(text: string): Promise<undefined> {
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

    _emitLocal(EventName.DIALOG, dialog)
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


export function _checkUpdate(): Promise<UpdateManifest> {
    return new Promise((resolve, reject) => {
        checkUpdate().then((res: UpdateResult) => {
            const { shouldUpdate, manifest } = res;
            if (shouldUpdate) {
                resolve(manifest!)
            } else {
                reject()
            }
        }).catch(e => {
            reject(e)
        })
    })
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
    }
    let message =  `New version <span onclick='_goBrowserPage("https://github.com/tzfun/etcd-workbench/releases/tag/App-${version}")' class="simulate-tag-a text-green font-weight-bold" title="Click to view updated content">${version}</span> released`

    if(timeDes) {
        message += ` ${timeDes}`
    }
    message += ', install it now?'
    return message
}

export function _checkUpdateAndInstall() {
    _loading(true, "Checking for update...")
    _checkUpdate().then(manifest => {
        _loading(false)
        let message = _genNewVersionUpdateMessage(manifest)

        _confirmUpdateApp(message).then(() => {
            _loading(true, "Installing...")
            installUpdate().then(() => {
                _loading(false)
                _loading(true, "Restarting...")
                relaunch().catch((e: string) => {
                    console.error(e)
                    _alertError("Unable to relaunch, please relaunch manually.")
                }).finally(() => {
                    _loading(false)
                })
            }).catch(e => {
                _loading(false)
                console.error(e)
                _alertError("Unable to update: " + e)
            })
        }).catch(() => {

        })
    }).catch((e) => {
        _loading(false)
        if (e == undefined) {
            _tipSuccess('Your version is already the latest')
        } else {
            _tipError(e)
        }
    })
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
