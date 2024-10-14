import {invoke} from "@tauri-apps/api";
import {ref} from "vue";
import {trackEvent} from "~/common/analytics.ts";

const platform = ref<PlatformType>()

export type PlatformType = 'linux' | 'darwin' | 'ios' | 'freebsd' | 'dragonfly' | 'netbsd' | 'openbsd' | 'solaris' | 'android' | 'win32' | string

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

export function _openMainWindow() {
    invoke('open_main_window').catch(e => {
        console.error(e)
    })
}

export function _openSettingWindow() {
    trackEvent('open_setting')
    invoke('open_setting_window').catch(e => {
        console.error(e)
    })
}

export function _exitApp() {
    invoke('exit_app').catch(e => {
        console.error(e)
    })
}

export function _openFolder(path: string, selectFile?: string): Promise<undefined> {
    return invoke('open_folder', {
        path,
        selectFile
    })
}

export function _getDownloadPath():Promise<string | undefined> {
    return invoke('get_download_path')
}


