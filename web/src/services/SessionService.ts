import request from '~/request'
import {NewSessionReq} from "~/entitys/RequestTypes";
import {host} from "~/Config";

export function newSession(data: NewSessionReq) {
    return request.post(host + "/session/new", data)
}

export function closeSession(sessionId: string) {
    return request.get(host + "/session/close", {sessionId: sessionId})
}

export function testSession(data: NewSessionReq): Promise<any> {
    return request.post(host + "/session/test", data)
}

export function heartBeat(sessionId: string): Promise<any> {
    return request.get(host + "/session/heart_beat", {sessionId: sessionId}, undefined, false, false)
}

export function getAllKeys(sessionId: string): Promise<any> {
    return request.get(host + "/session/etcd/kv/get_all_keys", {sessionId: sessionId})
}

export function getKV(sessionId: string, key: string, version?: number | null): Promise<any> {
    return request.get(host + "/session/etcd/kv/get", {
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
    return request.get(host + "/session/etcd/kv/get_history", {
        sessionId: sessionId,
        key: key,
        startVersion: startVersion,
        endVersion: endVersion
    })
}

export function deleteKey(sessionId: string, key: string): Promise<any> {
    return request.get(host + "/session/etcd/kv/delete", {
        sessionId: sessionId,
        key: key
    })
}

export function putKV(sessionId: string, key: string, value: string): Promise<any> {
    return request.get(host + "/session/etcd/kv/put", {
        sessionId: sessionId,
        key: key,
        value: value
    })
}
