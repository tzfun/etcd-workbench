import {SessionStoreConfig, SessionStoreConfigDict} from "~/entitys/TransformTypes";
import {deleteConfig, listConfig, saveConfig} from "~/services/SessionService";

export const host = import.meta.env.MODE === "test" ? "http://127.0.0.1:8002" : ""

let configDict: SessionStoreConfigDict = {}
let configDictListener: Function[] = []

export function clearAllConf() {
    configDict = {}
    notifyConfigChange()
}

function notifyConfigChange() {
    for (let listener of configDictListener) {
        listener()
    }
}

export function registerConfigListener(listener: Function) {
    configDictListener.push(listener)
}

export function unregisterConfigListener(listener: Function) {
    let idx = configDictListener.indexOf(listener)
    if (idx >= 0) {
        configDictListener.splice(idx, 1)
    }
}

export async function getAllConf(refresh: boolean): Promise<SessionStoreConfigDict> {
    if (refresh) {
        let list: SessionStoreConfig[] = await listConfig()
        let dict: SessionStoreConfigDict = {}
        for (let config of list) {
            dict[config.key!] = config
        }
        configDict = dict
        notifyConfigChange()
    }
    return configDict
}

export function deleteConf(key: string) {
    ElMessageBox.confirm(
        'Are you sure you want to delete this record?',
        'Confirm',
        {
            confirmButtonText: 'OK',
            cancelButtonText: 'Cancel',
            type: 'info',
        }
    ).then(() => {
        deleteConfig(key).then(() => {
            delete configDict[key]
        }).catch(e => {
            console.error(e)
        })
    }).catch(() => {
    })
}

export function saveConf(config: SessionStoreConfig) {
    saveConfig(config).then(data => {
        let key = data.data!
        config.key = key
        configDict[key] = config
    }).catch(e => {
        console.error(e)
    })
}
