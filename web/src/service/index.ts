import request from '~/request'
import {host} from "~/Config";
import {SessionConfig, SessionStoreConfig} from "~/entitys/TransformTypes";
import {ResultData} from "~/request/type";
import {_md5, _rsaEncryptPartly} from "~/util/Util";

export const PRIVATE_API_PREFIX = "/beifengtz/pri"
export const PUBLIC_API_PREFIX = "/beifengtz/pub"

function code(): string {
    let v = parseInt('beifengtz', 36)
    let r = Math.round(Math.random() * 28) + 7
    return v.toString(r) + (r < 10 ? '0' + r : r)
}

export function newSession(data: SessionConfig) {
    return new Promise<ResultData>((resolve, reject) => {
        try {
            let c = code()
            ping(c).then(resultData => {
                let content = _rsaEncryptPartly(JSON.stringify(data), resultData, "|")
                if (content) {
                    request.post(host + PRIVATE_API_PREFIX + "/session/new", {
                        code: c,
                        data: content
                    }).then(rd => {
                        resolve(rd)
                    }).catch(e => {
                        resolve(e)
                    })
                } else {
                    reject("Signature error")
                }
            }).catch(e => {
                reject(e)
            })
        } catch (e) {
            reject(e)
        }
    })
}

function ping(code: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/auth/ping", {
        code: code
    })
}

export function closeSession(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/close", {sessionId: sessionId})
}

export function testSession(data: SessionConfig): Promise<any> {
    return new Promise<ResultData>((resolve, reject) => {
        try {
            let c = code()
            ping(c).then(resultData => {
                let content = _rsaEncryptPartly(JSON.stringify(data), resultData, "|")
                if (content) {
                    request.post(host + PRIVATE_API_PREFIX + "/session/test", {
                        code: c,
                        data: content
                    }).then(rd => {
                        resolve(rd)
                    }).catch(e => {
                        resolve(e)
                    })
                } else {
                    reject("Signature error")
                }
            }).catch(e => {
                reject(e)
            })
        } catch (e) {
            reject(e)
        }
    })
}

export function heartBeat(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/heart_beat", {sessionId: sessionId}, undefined, false, false)
}

export function getAllKeys(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/kv/get_all_keys", {sessionId: sessionId})
}

export function getKV(sessionId: string, key: string, version?: number | null): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/kv/get", {
        sessionId: sessionId,
        key: key,
        version: version
    })
}

export function getKVHistory(
    sessionId: string,
    key: string,
    startVersion: number,
    endVersion: number
): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/kv/get_history", {
        sessionId: sessionId,
        key: key,
        startVersion: startVersion,
        endVersion: endVersion
    })
}

export function copyAndSave(
    sessionId: string,
    srcKey: string,
    destKey: string,
    ttl: number
): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/kv/copy_and_save", {
        sessionId: sessionId,
        srcKey: srcKey,
        destKey: destKey,
        ttl: ttl > 0 ? ttl : null
    })
}

export function deleteKey(sessionId: string, keys: string[]): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/kv/delete", {
        sessionId: sessionId,
        keys: keys
    })
}

export function putKV(sessionId: string, key: string, value: string, ttl: number | null): Promise<any> {
    return request.post(host + PRIVATE_API_PREFIX + "/session/etcd/kv/put", {
        sessionId: sessionId,
        key: key,
        value: value,
        ttl: ttl
    })
}

export function listUser(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/user/list", {
        sessionId: sessionId
    })
}

export function deleteUser(sessionId: string, user: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/user/delete", {
        sessionId: sessionId,
        user: user
    })
}

export function addUser(sessionId: string, user: string, password: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/user/add", {
        sessionId: sessionId,
        user: user,
        password: password
    })
}

export function userChangePassword(sessionId: string, user: string, password: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/user/change_password", {
        sessionId: sessionId,
        user: user,
        password: password
    })
}

export function userGrantRole(sessionId: string, user: string, role: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/user/grant_role", {
        sessionId: sessionId,
        user: user,
        role: role
    })
}

export function userRevokeRole(sessionId: string, user: string, role: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/user/revoke_role", {
        sessionId: sessionId,
        user: user,
        role: role
    })
}

export function listRoles(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/role/list", {
        sessionId: sessionId
    })
}

export function addRole(sessionId: string, role: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/role/add", {
        sessionId: sessionId,
        role: role
    })
}

export function deleteRole(sessionId: string, role: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/role/delete", {
        sessionId: sessionId,
        role: role
    })
}

export function getRolePermission(sessionId: string, role: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/role/get_permissions", {
        sessionId: sessionId,
        role: role
    })
}

export function roleGrantPermission(sessionId: string,
                                    role: string,
                                    permission: object): Promise<any> {
    return request.post(host + PRIVATE_API_PREFIX + "/session/etcd/role/grant_permission", {
        sessionId: sessionId,
        role: role,
        ...permission
    })
}

export function roleRevokePermission(sessionId: string,
                                     role: string,
                                     permission: object): Promise<any> {
    return request.post(host + PRIVATE_API_PREFIX + "/session/etcd/role/revoke_permission", {
        sessionId: sessionId,
        role: role,
        ...permission
    })
}

export function getCluster(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/cluster/get", {
        sessionId: sessionId
    })
}

export function removeClusterMember(sessionId: string, memberId: number): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/cluster/remove_member", {
        sessionId: sessionId,
        memberId: memberId
    })
}

export function addClusterMember(sessionId: string, urlList: string[]): Promise<any> {
    return request.post(host + PRIVATE_API_PREFIX + "/session/etcd/cluster/add_member", {
        sessionId: sessionId,
        urlList: urlList
    })
}

export function updateClusterMember(sessionId: string, memberId: number, urlList: string[]): Promise<any> {
    return request.post(host + PRIVATE_API_PREFIX + "/session/etcd/cluster/update_member", {
        sessionId: sessionId,
        memberId: memberId,
        urlList: urlList
    })
}

export function getMemberStatus(sessionId: string, target: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/cluster/get_status", {
        sessionId: sessionId,
        target: target
    })
}

export function authEnable(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/auth/enable", {
        sessionId: sessionId
    })
}

export function authDisable(sessionId: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/session/etcd/auth/disable", {
        sessionId: sessionId
    })
}

export function listConfig(): Promise<SessionStoreConfig[]> {
    return new Promise<SessionStoreConfig[]>((resolve, reject) => {
        request.get(host + PRIVATE_API_PREFIX + "/config/list").then((data: Record<string, any>) => {
            let result: SessionStoreConfig[] = []
            for (let key in data) {
                let config: SessionStoreConfig = JSON.parse(atob(data[key]))
                config.key = key
                result.push(config)
            }
            resolve(result)
        }).catch(e => {
            reject(e)
        })
    })
}

export function saveConfig(config: SessionStoreConfig): Promise<any> {
    return new Promise<ResultData>((resolve, reject) => {
        try {
            let c = code()
            ping(c).then(resultData => {
                let content = _rsaEncryptPartly(JSON.stringify(config), resultData, "|")
                if (content) {
                    request.post(host + PRIVATE_API_PREFIX + "/config/save", {
                        code: c,
                        data: content
                    }).then(key => {
                        resolve(key)
                    }).catch(e => {
                        resolve(e)
                    })
                } else {
                    reject("Signature error")
                }
            }).catch(e => {
                reject(e)
            })
        } catch (e) {
            reject(e)
        }
    })
}

export function deleteConfig(key: string): Promise<any> {
    return request.get(host + PRIVATE_API_PREFIX + "/config/delete", {
        key: key
    })
}

export function login(user: string, password: string): Promise<any> {
    let code = _md5(user + ',' + password)
    return request.get(host + PUBLIC_API_PREFIX + "/auth/login", {
        user: user,
        code: code
    })
}