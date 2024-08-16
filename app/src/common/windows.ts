import {WebviewWindow} from "@tauri-apps/api/window";


export async function _openMainWindow() {
    let splashscreenWindow = WebviewWindow.getByLabel('splashscreen')
    if(splashscreenWindow) {
        await splashscreenWindow.close()
    }
    let mainWindow = WebviewWindow.getByLabel('main')
    if (mainWindow) {
        await mainWindow.show()
    }
}


export async function _openSettingWindow() {
    let settingWindow = WebviewWindow.getByLabel('setting')
    if (settingWindow) {
        if (await settingWindow.isVisible()) {
            await settingWindow.setFocus()
        } else {
            await settingWindow.show()
        }
    }
}