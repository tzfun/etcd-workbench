import {invoke} from "@tauri-apps/api";
import {ErrorPayload} from "./transport/connection";

export type CustomUpdateManifest = {
    version: string
    body: string
    date?: number
    source: string
}

export function _checkUpdate(): Promise<boolean> {
    return new Promise((resolve, reject) => {
        invoke<boolean>("check_update").then(data => {
            resolve(data)
        }).catch(e => {
            reject((e as ErrorPayload).errMsg)
        })
    })
}

export function _installUpdate(): Promise<void> {
    return new Promise((resolve, reject) => {
        invoke("install_update").then(() => {
            resolve()
        }).catch(e => {
            reject((e as ErrorPayload).errMsg)
        })
    })
}