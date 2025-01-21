import {invoke} from "@tauri-apps/api";
import {ref} from "vue";
import {trackEvent} from "~/common/analytics.ts";
import {appWindow} from "@tauri-apps/api/window";
import {ask} from "@tauri-apps/api/dialog";
import { _emitWindow, EventName } from "./events";

const platform = ref<PlatformType>()
export const isMaximizeState = ref<boolean>(false)

export type PlatformType =
    'linux'
    | 'darwin'
    | 'ios'
    | 'freebsd'
    | 'dragonfly'
    | 'netbsd'
    | 'openbsd'
    | 'solaris'
    | 'android'
    | 'win32'
    | string

export function _setPlatform(p: PlatformType) {
    platform.value = p
}

export function _isWindows(): boolean {
    return platform.value == 'win32'
}

export function _isMac(): boolean {
    return platform.value == 'darwin'
}

export function _isLinux(): boolean {
    return platform.value == 'linux'
}

export function _updateMaximizeState() {
    appWindow.isMaximized().then(state => {
        isMaximizeState.value = state
    })
}

export function _onClientError(info: string, err: string, exitAppFinally: boolean = false) {
    let message = "An error occurred. To help resolve the issue, do you want to report it to the author?"
    if (exitAppFinally) {
        message += `\n\n(App will exit later)`
    }
    ask(message, {
        title: "System",
        type: "error"
    }).then(sure => {
        if (sure) {
            trackEvent("client_error", {
                client_err_info: info,
                client_err_stack: err
            }).then(() => {
            })
        }
    }).finally(() => {
        if (exitAppFinally) {
            _exitApp()
        }
    })
    invoke('client_error', {
        info: info || '',
        err: err || ''
    }).catch(e => {
        console.error(e)
    })
}

export function _openMainWindow() {
    invoke('open_main_window').catch(e => {
        console.error(e)
    })
}

export function _openSettingWindow(anchor?: string) {
    trackEvent('open_setting')
    invoke('open_setting_window').then(() => {
        if(anchor) {
            _emitWindow('setting', EventName.SET_SETTING_ANCHOR, anchor)
        }
    }).catch(e => {
        console.error(e)
    })
}

export function _exitApp(): Promise<unknown> {
    return invoke('exit_app').catch(e => {
        console.error(e)
    })
}

export function _openFolder(path: string, selectFile?: string): Promise<undefined> {
    return invoke('open_folder', {
        path,
        selectFile
    })
}

export function _getDownloadPath(): Promise<string | undefined> {
    return invoke('get_download_path')
}


