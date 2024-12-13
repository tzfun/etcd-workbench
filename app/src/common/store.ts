import {
    DEFAULT_GLOBAL_STORE,
    DEFAULT_SETTING_CONFIG,
    GlobalStoreConfig,
    SettingConfig,
    UpdateInfo
} from "~/common/transport/setting.ts";
import {Ref, ref, UnwrapRef} from "vue";
import {invoke} from "@tauri-apps/api";
import {_getAppVersion} from "~/common/services.ts";
import {EditorHighlightLanguage} from "~/common/types.ts";

const SETTINGS = ref<SettingConfig>(DEFAULT_SETTING_CONFIG)
const GLOBAL_STORE = ref<GlobalStoreConfig>(DEFAULT_GLOBAL_STORE)
const updateInfo = ref<UpdateInfo>({
    valid: false
})
const appVersion = ref<string>("0.0.0")

export function _useSettings(): Ref<UnwrapRef<SettingConfig>> {
    return SETTINGS
}

export function _setLocalSettings(settingConfig: SettingConfig) {
    SETTINGS.value = settingConfig
}

export function _loadSettings(): Promise<SettingConfig> {
    return new Promise((resolve, reject) => {
        invoke('get_settings').then(data => {
            SETTINGS.value = data as SettingConfig
            resolve(SETTINGS.value)
        }).catch(e => {
            reject(e)
        })
    })
}

export function _saveSettings(settingConfig: SettingConfig) {
    invoke('save_settings', {
        settingConfig
    }).then(() => {
        SETTINGS.value = settingConfig
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

export function _useGlobalStore(): Ref<UnwrapRef<GlobalStoreConfig>> {
    return GLOBAL_STORE
}

export function _loadGlobalStore(): Promise<GlobalStoreConfig> {
    return new Promise((resolve, reject) => {
        invoke('get_global_store').then(data => {
            let store = data as GlobalStoreConfig
            let map:Record<string, EditorHighlightLanguage> = {}
            store.fileFormatLog.forEach(({key, format}) => {
                map[key] = format
            })
            store.fileFormatLogMap = map
            GLOBAL_STORE.value = store
            resolve(GLOBAL_STORE.value)
        }).catch(e => {
            reject(e)
        })
    })
}

export function _saveGlobalStore(store: GlobalStoreConfig) {
    invoke('save_global_store', {
        store
    }).then(() => {
        GLOBAL_STORE.value = store
    }).catch(e => {
        console.error(e)
    })
}