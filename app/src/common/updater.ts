import { invoke } from "@tauri-apps/api";
import { relaunch } from "@tauri-apps/api/process";
import { checkUpdate, installUpdate, UpdateManifest, UpdateResult } from "@tauri-apps/api/updater";
import { _alertError, _confirmUpdateApp, _genNewVersionUpdateMessage, _loading, _tipError, _tipSuccess } from "./events";
import { ErrorPayload } from "./transport/connection";

export type CustomUpdateManifest = {
    version: string
    body: string
    date?: number
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

export function _checkUpdateAndInstall() {
    _loading(true, "Checking for update...")

    let checkPromise: Promise<UpdateManifest> = new Promise((resolve, reject) => {
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

    checkPromise.then(manifest => {
        _loading(false)
        let message = _genNewVersionUpdateMessage(manifest)

        _confirmUpdateApp(message).then(() => {
            _loading(true, "Installing...")
            _installUpdate().then(() => {
                _loading(false)
                _loading(true, "Restarting...")
                relaunch().catch((e: string) => {
                    console.error(e)
                    _alertError("Unable to relaunch, please relaunch manually.")
                }).finally(() => {
                    _loading(false)
                })
            }).catch(e => {
                _loading(false)
                console.error(e)
                _alertError("Unable to update: " + e)
            })
        }).catch(() => {

        })
    }).catch((e) => {
        _loading(false)
        if (e == undefined) {
            _tipSuccess('Your version is already the latest')
        } else {
            _tipError(e)
        }
    })
}

function checkUpdateNative(): Promise<UpdateManifest> {
    return new Promise((resolve, reject) => {
        checkUpdate().then((res: UpdateResult) => {
            const { shouldUpdate, manifest } = res;
            if (shouldUpdate) {
                resolve(manifest!)
            } else {
                reject()
            }
        }).catch(e => {
            reject(e)
        })
    })
}

function checkUpdateCustom(): Promise<UpdateManifest> {
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