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

export function _closeAllWindow() {
    invoke('close_all_window').catch(e => {
        console.error(e)
    })
}