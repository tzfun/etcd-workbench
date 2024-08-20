import {DEFAULT_SETTING_CONFIG, SettingConfig} from "~/common/transport/setting.ts";
import {Ref, ref, UnwrapRef} from "vue";
import {invoke} from "@tauri-apps/api";

const settings = ref<SettingConfig>(DEFAULT_SETTING_CONFIG)

export function _useSettings(): Ref<UnwrapRef<SettingConfig>> {
    return settings
}

export function _setLocalSettings(settingConfig: SettingConfig) {
    settings.value = settingConfig
}

export function _loadSettings(): Promise<SettingConfig> {
    return new Promise((resolve, reject) => {
        invoke('get_settings').then(data => {
            settings.value = data as SettingConfig
            resolve(settings.value)
        }).catch(e => {
            reject(e)
        })
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
