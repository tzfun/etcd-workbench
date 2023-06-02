import request from '~/request'
import {NewSessionReq} from "~/entitys/RequestTypes";

const host = "http://127.0.0.1:8002"

export function newSession(data: NewSessionReq) {
    return request.post(host + "/session/new", data)
}

export function closeSession(key: string) {
    return request.get(host + "/session/close", {key: key})
}

export function testSession(data: NewSessionReq): Promise<any> {
    return request.post(host + "/session/test", data)
}

export function heartBeat(key: string): Promise<any> {
    return request.get(host + "/session/heart_beat", {key: key})
}

export function getAllKeys(key: string): Promise<any> {
    return request.get(host + "/session/get_all_keys", {key: key})
}