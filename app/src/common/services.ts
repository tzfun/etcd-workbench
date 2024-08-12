import {invoke} from "@tauri-apps/api";
import {Connection, ConnectionInfo, SessionData} from "~/common/transport/connection.ts";
import {Cluster} from "~/common/transport/maintenance.ts";
import {KeyValue} from "~/common/transport/kv.ts";

export function _connectTest(connection: Connection): Promise<undefined> {
    return invoke('connect_test', {connection})
}

export function _connect(connection: Connection): Promise<SessionData> {
    return invoke('connect', {connection: connection})
}

export function _disconnect(sessionId: number): Promise<undefined> {
    return invoke('disconnect', {session: sessionId})
}

export function _getConnectionList(): Promise<ConnectionInfo[]> {
    return invoke('get_connection_list')
}

export function _saveConnection(connection: ConnectionInfo): Promise<undefined> {
    return invoke("save_connection", {connection: connection})
}

export function _removeConnection(name: string): Promise<undefined> {
    return invoke("remove_connection", {name: name})
}

export function _getCluster(sessionId: number): Promise<Cluster> {
    return invoke('get_cluster', {session: sessionId})
}

export function _getAllKeys(sessionId: number): Promise<KeyValue[]> {
    return invoke('kv_get_all_keys', {session: sessionId})
}

export function _getKV(sessionId: number, key: string): Promise<KeyValue> {
    return invoke('kv_get', {
        session: sessionId,
        key
    })
}

export function _getKVByVersion(sessionId: number, key: string, version: number): Promise<KeyValue> {
    return invoke('kv_get_by_version', {
        session: sessionId,
        key,
        version
    })
}

export function _putKV(sessionId: number, key: string, value: number[], ttl?: string): Promise<undefined> {
    return invoke('kv_put', {
        session: sessionId,
        key,
        value,
        ttl
    })
}

export function _deleteKV(sessionId: number, keys: string[]): Promise<number> {
    return invoke('kv_delete', {
        session: sessionId,
        keys
    })
}

export function _getKVHistoryVersions(sessionId: number, key: string, start: number, end: number): Promise<number[]> {
    return invoke('kv_get_history_versions', {
        session: sessionId,
        key,
        start,
        end
    })
}