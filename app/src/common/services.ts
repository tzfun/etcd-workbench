import {invoke} from "@tauri-apps/api";
import {Connection, ConnectionInfo, SessionData} from "~/common/transport/connection.ts";
import {Cluster} from "~/common/transport/maintenance.ts";
import {KeyValue, LeaseInfo} from "~/common/transport/kv.ts";
import {_tipError, events} from "~/common/events.ts";
import {LogicErrorInfo} from "~/common/types.ts";

export function _handleError(info: LogicErrorInfo) {
    let error = info.e
    console.error(error)

    if (typeof error === 'string') {
        _tipError((info.prefix ? info.prefix : "") + info.e)
    } else {
        _tipError((info.prefix ? info.prefix : "") + error.errMsg)
        if (error.errType == "Unauthenticated" && info.session) {
            events.emit('closeTab', info.session.id)
        }
    }
}

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

export function _putKV(sessionId: number, key: string, value: number[], ttl?: number): Promise<undefined> {
    return invoke('kv_put', {
        session: sessionId,
        key,
        value,
        ttl
    })
}

export function _putKVWithLease(sessionId: number, key: string, value: number[], lease: string): Promise<undefined> {
    return invoke('kv_put_with_lease', {
        session: sessionId,
        key,
        value,
        lease
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

export function _getLease(sessionId: number, lease: string): Promise<LeaseInfo> {
    return invoke('lease_get', {
        session: sessionId,
        lease,
    })
}

export function _leases(sessionId: number): Promise<string[]> {
    return invoke('leases', {
        session: sessionId
    })
}

export function _revokeLeases(sessionId: number, lease: string): Promise<undefined> {
    return invoke('lease_revoke', {
        session: sessionId,
        lease
    })
}

export function _grantLease(sessionId: number, ttl: number, lease?: string): Promise<string> {
    return invoke('lease_grant', {
        session: sessionId,
        ttl,
        lease
    })
}