import {DEFAULT_SETTING_CONFIG, SettingConfig} from "~/common/transport/setting.ts";
import {ref} from "vue";
import {invoke} from "@tauri-apps/api";

const settings = ref<SettingConfig>(DEFAULT_SETTING_CONFIG)

export function _getSettings(): SettingConfig {
    return settings.value
}

export function _loadSettings(): Promise<undefined> {
    return invoke('get_settings').then(data => {
        settings.value = data as SettingConfig
    }).catch(e => {
        console.log(e)
    })
}

export function _saveSettings(settingConfig: SettingConfig) {
    invoke('save_settings', {
        settingConfig
    }).then(() => {
        settings.value = settingConfig
    }).catch(e => {
        console.error(e)
    })
}
