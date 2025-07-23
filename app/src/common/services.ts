import {invoke} from "@tauri-apps/api";
import {Connection, ConnectionInfo, KeyMonitorConfig, SessionData} from "~/common/transport/connection.ts";
import {Cluster, SnapshotInfo} from "~/common/transport/maintenance.ts";
import {KeyValue, KVPutResult, LeaseInfo, PutStrategy, SearchResult} from "~/common/transport/kv.ts";
import {_emitLocal, _tipError, EventName} from "~/common/events.ts";
import {LogicErrorInfo} from "~/common/types.ts";
import {RolePermission, User} from "~/common/transport/user.ts";

export function _handleError(info: LogicErrorInfo) {
    let error = info.e
    console.error(error)

    if (typeof error === 'string') {
        _tipError((info.prefix ? info.prefix : "") + info.e)
    } else {
        _tipError((info.prefix ? info.prefix : "") + error.errMsg)
        if (error.errType == "Unauthenticated" && info.session) {
            _emitLocal(EventName.CLOSE_TAB, info.session.id)
        }
    }
}

export function _getAppVersion(): Promise<string> {
    return invoke('get_app_version')
}

export function _isDebugModel(): Promise<boolean> {
    return invoke('is_debug_model')
}

export function _connectTest(connection: Connection): Promise<undefined> {
    return invoke('connect_test', {connection})
}

export function _connect(name: string, connection: Connection): Promise<SessionData> {
    return invoke('connect', {
        name,
        connection
    })
}

export function _disconnect(sessionId: number): Promise<undefined> {
    return invoke('disconnect', {session: sessionId})
}

export function _getConnectionList(): Promise<ConnectionInfo[]> {
    return invoke('get_connection_list')
}

export function _saveConnection(name: string, connection: Connection): Promise<undefined> {
    return invoke("save_connection", {
        name,
        connection
    })
}

export function _removeConnection(name: string): Promise<undefined> {
    return invoke("remove_connection", {name: name})
}

export function _exportConnection(filepath: string): Promise<undefined> {
    return invoke('export_connection', {filepath: filepath})
}

export function _importConnection(filepath: string): Promise<undefined> {
    return invoke('import_connection', {filepath: filepath})
}

export function _getCluster(sessionId: number): Promise<Cluster> {
    return invoke('get_cluster', {session: sessionId})
}

export function _defragment(sessionId: number): Promise<undefined> {
    return invoke('maintenance_defragment', {session: sessionId})
}

export function _compact(sessionId: number, revision: number, physical: boolean): Promise<undefined> {
    return invoke('maintenance_compact', {session: sessionId, revision, physical})
}

export function _metrics(sessionId: number): Promise<Array<string[]>> {
    return invoke<Array<string[]>>('metrics', {session: sessionId})
}

export function _getAllKeys(sessionId: number): Promise<KeyValue[]> {
    return invoke('kv_get_all_keys', {session: sessionId})
}

export function _getAllKeysPaging(sessionId: number, cursorKey: string, limit: number): Promise<KeyValue[]> {
    return invoke('kv_get_all_keys_paging', {
        session: sessionId,
        cursorKey,
        limit
    })
}

export function _getKV(sessionId: number, key: string, keyBytes?: number[]): Promise<KeyValue> {
    return invoke('kv_get', {
        session: sessionId,
        key,
        keyBytes
    })
}

export function _getKVByVersion(sessionId: number, key: string, version: number, keyBytes?: number[]): Promise<KeyValue> {
    return invoke('kv_get_by_version', {
        session: sessionId,
        key,
        keyBytes,
        version
    })
}

export function _searchByPrefix(sessionId: number, prefix: string): Promise<SearchResult> {
    return invoke('kv_get_with_prefix', {
        session: sessionId,
        prefix
    })
}

/**
 * 插入/更新一个新的key
 *
 * @param sessionId 会话ID
 * @param key 插入的Key
 * @param value 插入的值，字节数组
 * @param version 客户端读取的最新版本号，如果 >=0 则会进行冲突判断，如果 <0 则不判断冲突直接插入
 * @param ttl key过期时间
 */
export function _putKV(sessionId: number, key: string, value: number[], version: number, ttl?: number): Promise<KVPutResult> {
    return invoke('kv_put', {
        session: sessionId,
        key,
        value,
        version,
        ttl
    })
}

export function _putKVWithLease(sessionId: number, key: string, value: number[], lease: string): Promise<void> {
    return invoke('kv_put_with_lease', {
        session: sessionId,
        key,
        value,
        lease
    })
}

/**
 * 删除Key。最终会删除 keys 和 keyBytes 中所有的key
 *
 * @param sessionId 会话ID
 * @param utf8EncodedKeys 可 UTF8 编码的key数组
 * @param unUtf8EncodedKeys 无法 UTF8 编码的key数组
 */
export function _deleteKV(sessionId: number, utf8EncodedKeys: string[], unUtf8EncodedKeys: number[][]): Promise<number> {
    return invoke('kv_delete', {
        session: sessionId,
        keys: utf8EncodedKeys,
        keyBytes: unUtf8EncodedKeys
    })
}

export function _getKVHistoryVersions(sessionId: number, key: string, start: number, end: number, keyBytes?: number[]): Promise<number[]> {
    return invoke('kv_get_history_versions', {
        session: sessionId,
        key,
        start,
        end,
        keyBytes
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

export function _getAllUsers(sessionId: number): Promise<User[]> {
    return invoke('user_list', {
        session: sessionId,
    })
}

export function _addUser(sessionId: number, user: string, password: string): Promise<undefined> {
    return invoke('user_add', {
        session: sessionId,
        user,
        password
    })
}

export function _deleteUser(sessionId: number, user: string): Promise<undefined> {
    return invoke('user_delete', {
        session: sessionId,
        user
    })
}

export function _userChangePassword(sessionId: number, user: string, newPassword: string): Promise<undefined> {
    return invoke('user_change_password', {
        session: sessionId,
        user,
        newPassword
    })
}

export function _userGrantRole(sessionId: number, user: string, role: string): Promise<undefined> {
    return invoke('user_grant_role', {
        session: sessionId,
        user,
        role
    })
}

export function _userRevokeRole(sessionId: number, user: string, role: string): Promise<undefined> {
    return invoke('user_revoke_role', {
        session: sessionId,
        user,
        role
    })
}

export function _authEnable(sessionId: number): Promise<undefined> {
    return invoke('auth_enable', {
        session: sessionId,
    })
}

export function _authDisable(sessionId: number): Promise<undefined> {
    return invoke('auth_disable', {
        session: sessionId,
    })
}

export function _getAllRoles(sessionId: number): Promise<string[]> {
    return invoke('role_list', {
        session: sessionId
    })
}

export function _getRolePermissions(sessionId: number, role: string): Promise<RolePermission[]> {
    return invoke('role_get_permissions', {
        session: sessionId,
        role
    })
}

export function _deleteRole(sessionId: number, role: string): Promise<undefined> {
    return invoke('role_delete', {
        session: sessionId,
        role
    })
}

export function _addRole(sessionId: number, role: string): Promise<undefined> {
    return invoke('role_add', {
        session: sessionId,
        role
    })
}

export function _grantRolePermissions(sessionId: number, role: string, permission: RolePermission): Promise<undefined> {
    return invoke('role_grant_permission', {
        session: sessionId,
        role,
        permission
    })
}

export function _revokeRolePermissions(sessionId: number, role: string, permission: RolePermission): Promise<undefined> {
    return invoke('role_revoke_permission', {
        session: sessionId,
        role,
        permission
    })
}

export function _maintenanceCreateSnapshotTask(sessionId: number, filepath: string): Promise<SnapshotInfo> {
    return invoke('maintenance_create_snapshot_task', {
        session: sessionId,
        filepath
    })
}

export function _maintenanceStopSnapshotTask(taskId: number): Promise<undefined> {
    return invoke('maintenance_stop_snapshot_task', {
        taskId
    })
}

export function _maintenanceRemoveSnapshotTask(taskId: number): Promise<undefined> {
    return invoke('maintenance_remove_snapshot_task', {
        taskId
    })
}

export function _maintenanceListSnapshotTask(): Promise<SnapshotInfo[]> {
    return invoke('maintenance_list_snapshot_task')
}

export function _updateKeyCollection(session: number, keyCollection: string[]): Promise<undefined> {
    return invoke('update_key_collection', {
        session,
        keyCollection
    })
}

export function _setKeyMonitor(session: number, keyMonitor: KeyMonitorConfig): Promise<undefined> {
    return invoke('set_key_monitor', {
        session,
        keyMonitor
    })
}

export function _removeKeyMonitor(session: number, key: string): Promise<undefined> {
    return invoke('remove_key_monitor', {
        session,
        key
    })
}

export function _kvSearchNextDir(session: number, prefix: string, includeFile: boolean): Promise<string[]> {
    return invoke<string[]>('kv_search_next_dir', {
        session,
        prefix,
        includeFile
    })
}

export function _kvRenameDir(
    session: number,
    originPrefix: string,
    newPrefix: string,
    deleteOriginKeys: boolean,
    putStrategy: PutStrategy
): Promise<string[]> {
    return invoke<string[]>('kv_rename_dir', {
        session,
        originPrefix,
        newPrefix,
        deleteOriginKeys,
        putStrategy
    })
}