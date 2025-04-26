import {invoke} from "@tauri-apps/api";
import {UpdateManifest} from "@tauri-apps/api/updater";
import {ErrorPayload} from "./transport/connection";

export type CustomUpdateManifest = {
    version: string
    body: string
    date?: number
    source: string
}

export function _checkUpdate(): Promise<UpdateManifest> {
    return new Promise((resolve, reject) => {
        invoke("check_update").then(data => {
            if (data) {
                resolve(data as UpdateManifest)
            } else {
                reject(undefined)
            }
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