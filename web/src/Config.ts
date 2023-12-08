import {SessionStoreConfig, SessionStoreConfigDict} from "~/entitys/TransformTypes";
import {deleteConfig, listConfig, saveConfig} from "~/service";

export const host = import.meta.env.MODE === "test" ? "http://127.0.0.1:8002" : ""

let configDict: SessionStoreConfigDict = {}
let configDictListener: Function[] = []

export function clearAllConf() {
    configDict = {}
    onDirty()
}

function onDirty() {
    for (let listener of configDictListener) {
        listener()
    }
}

export function registerConfigListener(listener: Function) {
    configDictListener.push(listener)
}

export function unregisterConfigListener(listener: Function | undefined) {
    if (listener) {
        let idx = configDictListener.indexOf(listener)
        if (idx >= 0) {
            configDictListener.splice(idx, 1)
        }
    }
}

export function loadConfAsync(): Promise<SessionStoreConfigDict> {
    return new Promise((resolve, reject) => {
        listConfig().then(list => {
            let dict: SessionStoreConfigDict = {}
            for (let config of list) {
                dict[config.key!] = config
            }
            configDict = dict
            onDirty()
            resolve(dict)
        }).catch(e => {
            reject(e)
        })
    })
}

export function getAllConf(): SessionStoreConfigDict {
    return configDict
}

export function deleteConf(key: string) {
    ElMessageBox.confirm(
        'Are you sure you want to delete this record?',
        'Confirm',
        {
            confirmButtonText: 'Yes',
            cancelButtonText: 'Cancel',
            type: 'info',
        }
    ).then(() => {
        deleteConfig(key).then(() => {
            delete configDict[key]
            onDirty()
        }).catch(e => {
            console.error(e)
        })
    }).catch(() => {
    })
}

export function saveConf(config: SessionStoreConfig) {
    saveConfig(config).then(key => {
        config.key = key
        configDict[key] = config
        onDirty()
    }).catch(e => {
        console.error(e)
    })
}
