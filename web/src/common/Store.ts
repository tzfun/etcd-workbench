import {clearAllConf} from "~/common/Config";
import {pushEvent} from "~/common/Event";

let data = {
    user: <string | null>null,
    token: <string | null>null
}

export function loadStore() {
    let content = window.localStorage.getItem("etcd-workbench")
    if (content) {
        data = JSON.parse(content)
    }
}

function onDirty() {
    window.localStorage.setItem("etcd-workbench", JSON.stringify(data))
    pushEvent("storeChange")
}

export function getToken() {
    return data.token
}

export function setToken(token: string | null) {
    data.token = token
    onDirty()
}

export function getUser() {
    return data.user
}

export function setUser(user: string | null) {
    data.user = user
    onDirty()
}

export function isLogin() {
    return data.user && data.token
}

export function clearLoginStatus() {
    data.user = null
    data.token = null
    clearAllConf()
    onDirty()
}

loadStore()
