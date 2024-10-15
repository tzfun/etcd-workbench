import {DEFAULT_SETTING_CONFIG, SettingConfig, UpdateInfo} from "~/common/transport/setting.ts";
import {Ref, ref, UnwrapRef} from "vue";
import {invoke} from "@tauri-apps/api";
import {_getAppVersion} from "~/common/services.ts";

const settings = ref<SettingConfig>(DEFAULT_SETTING_CONFIG)
const updateInfo = ref<UpdateInfo>({
    valid: false
})
const appVersion = ref<string>("0.0.0")

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

export function _useUpdateInfo(): Ref<UnwrapRef<UpdateInfo>> {
    return updateInfo
}

export function _loadAppVersion(): Promise<string> {
    return new Promise<string>(resolve => {
        _getAppVersion().then((version: string) => {
            appVersion.value = version
            resolve(version)
        }).catch(e => {
            console.error(e)
            resolve(appVersion.value)
        })
    })
}

export function _getAppVersionCache(): string {
    return appVersion.value
}