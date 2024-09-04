import {invoke} from "@tauri-apps/api";

export function _openMainWindow() {
    invoke('open_main_window').catch(e => {
        console.error(e)
    })
}

export function _openSettingWindow() {
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

export function _getDownloadPath():Promise<string | null> {
    return invoke('get_download_path')
}