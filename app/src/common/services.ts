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
        _tipError((info.prefix ? info.prefix : "Error: ") + info.e)
    } else {
        _tipError((info.prefix ? info.prefix : "Error: ") + error.errMsg)
        if (error.errType == "Unauthenticated" && info.session) {
            _emitLocal(EventName.CLOSE_TAB, info.session.id)
        }
    }
}

export function _getAppVersion() {
    return invoke<string>('get_app_version')
}

export function _isDebugModel() {
    return invoke<boolean>('is_debug_model')
}

export function _connectTest(connection: Connection) {
    return invoke('connect_test', {connection})
}

export function _connect(name: string, connection: Connection) {
    return invoke<SessionData>('connect', {
        name,
        connection
    })
}

export function _disconnect(sessionId: number) {
    return invoke('disconnect', {session: sessionId})
}

export function _getConnectionList() {
    return invoke<ConnectionInfo[]>('get_connection_list')
}

export function _saveConnection(name: string, connection: Connection) {
    return invoke("save_connection", {
        name,
        connection
    })
}

export function _removeConnection(name: string) {
    return invoke("remove_connection", {name: name})
}

export function _exportConnection(filepath: string) {
    return invoke('export_connection', {filepath: filepath})
}

export function _importConnection(filepath: string) {
    return invoke('import_connection', {filepath: filepath})
}

export function _getCluster(sessionId: number) {
    return invoke<Cluster>('get_cluster', {session: sessionId})
}

export function _defragment(sessionId: number) {
    return invoke('maintenance_defragment', {session: sessionId})
}

export function _compact(sessionId: number, revision: number, physical: boolean) {
    return invoke('maintenance_compact', {session: sessionId, revision, physical})
}

export function _metrics(sessionId: number) {
    return invoke<Array<string[]>>('metrics', {session: sessionId})
}

export function _getAllKeys(sessionId: number) {
    return invoke<KeyValue[]>('kv_get_all_keys', {session: sessionId})
}

export function _getAllKeysPaging(sessionId: number, cursorKey: string, limit: number) {
    return invoke<KeyValue[]>('kv_get_all_keys_paging', {
        session: sessionId,
        cursorKey,
        limit
    })
}

export function _getKV(sessionId: number, key: string, keyBytes?: number[]) {
    return invoke<KeyValue>('kv_get', {
        session: sessionId,
        key,
        keyBytes
    })
}

export function _getKVByVersion(sessionId: number, key: string, version: number, keyBytes?: number[]) {
    return invoke<KeyValue>('kv_get_by_version', {
        session: sessionId,
        key,
        keyBytes,
        version
    })
}

export function _searchByPrefix(sessionId: number, prefix: string) {
    return invoke<SearchResult>('kv_get_with_prefix', {
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
export function _putKV(sessionId: number, key: string, value: number[], version: number, ttl?: number) {
    return invoke<KVPutResult>('kv_put', {
        session: sessionId,
        key,
        value,
        version,
        ttl
    })
}

export function _putKVWithLease(sessionId: number, key: string, value: number[], lease: string) {
    return invoke<void>('kv_put_with_lease', {
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
export function _deleteKV(sessionId: number, utf8EncodedKeys: string[], unUtf8EncodedKeys: number[][]) {
    return invoke<number>('kv_delete', {
        session: sessionId,
        keys: utf8EncodedKeys,
        keyBytes: unUtf8EncodedKeys
    })
}

export function _getKVHistoryVersions(sessionId: number, key: string, start: number, end: number, keyBytes?: number[]) {
    return invoke<number[]>('kv_get_history_versions', {
        session: sessionId,
        key,
        start,
        end,
        keyBytes
    })
}

export function _getLease(sessionId: number, lease: string) {
    return invoke<LeaseInfo>('lease_get', {
        session: sessionId,
        lease,
    })
}

export function _leases(sessionId: number) {
    return invoke<string[]>('leases', {
        session: sessionId
    })
}

export function _revokeLeases(sessionId: number, lease: string) {
    return invoke('lease_revoke', {
        session: sessionId,
        lease
    })
}

export function _grantLease(sessionId: number, ttl: number, lease?: string) {
    return invoke<string>('lease_grant', {
        session: sessionId,
        ttl,
        lease
    })
}

export function _getAllUsers(sessionId: number) {
    return invoke<User[]>('user_list', {
        session: sessionId,
    })
}

export function _addUser(sessionId: number, user: string, password: string) {
    return invoke('user_add', {
        session: sessionId,
        user,
        password
    })
}

export function _deleteUser(sessionId: number, user: string) {
    return invoke('user_delete', {
        session: sessionId,
        user
    })
}

export function _userChangePassword(sessionId: number, user: string, newPassword: string) {
    return invoke('user_change_password', {
        session: sessionId,
        user,
        newPassword
    })
}

export function _userGrantRole(sessionId: number, user: string, role: string) {
    return invoke('user_grant_role', {
        session: sessionId,
        user,
        role
    })
}

export function _userRevokeRole(sessionId: number, user: string, role: string) {
    return invoke('user_revoke_role', {
        session: sessionId,
        user,
        role
    })
}

export function _authEnable(sessionId: number) {
    return invoke('auth_enable', {
        session: sessionId,
    })
}

export function _authDisable(sessionId: number) {
    return invoke('auth_disable', {
        session: sessionId,
    })
}

export function _getAllRoles(sessionId: number): Promise<string[]> {
    return invoke('role_list', {
        session: sessionId
    })
}

export function _getRolePermissions(sessionId: number, role: string) {
    return invoke<RolePermission[]>('role_get_permissions', {
        session: sessionId,
        role
    })
}

export function _deleteRole(sessionId: number, role: string) {
    return invoke('role_delete', {
        session: sessionId,
        role
    })
}

export function _addRole(sessionId: number, role: string) {
    return invoke('role_add', {
        session: sessionId,
        role
    })
}

export function _grantRolePermissions(sessionId: number, role: string, permission: RolePermission) {
    return invoke('role_grant_permission', {
        session: sessionId,
        role,
        permission
    })
}

export function _revokeRolePermissions(sessionId: number, role: string, permission: RolePermission) {
    return invoke('role_revoke_permission', {
        session: sessionId,
        role,
        permission
    })
}

export function _maintenanceCreateSnapshotTask(sessionId: number, filepath: string) {
    return invoke<SnapshotInfo>('maintenance_create_snapshot_task', {
        session: sessionId,
        filepath
    })
}

export function _maintenanceStopSnapshotTask(taskId: number) {
    return invoke('maintenance_stop_snapshot_task', {
        taskId
    })
}

export function _maintenanceRemoveSnapshotTask(taskId: number) {
    return invoke('maintenance_remove_snapshot_task', {
        taskId
    })
}

export function _maintenanceListSnapshotTask() {
    return invoke<SnapshotInfo[]>('maintenance_list_snapshot_task')
}

export function _updateKeyCollection(session: number, keyCollection: string[]) {
    return invoke('update_key_collection', {
        session,
        keyCollection
    })
}

export function _setKeyMonitor(session: number, keyMonitor: KeyMonitorConfig) {
    return invoke('set_key_monitor', {
        session,
        keyMonitor
    })
}

export function _removeKeyMonitor(session: number, key: string) {
    return invoke('remove_key_monitor', {
        session,
        key
    })
}

export function _kvSearchNextDir(session: number, prefix: string, includeFile: boolean) {
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
) {
    return invoke<string[]>('kv_rename_dir', {
        session,
        originPrefix,
        newPrefix,
        deleteOriginKeys,
        putStrategy
    })
}

export function _kvBatchExport(session: number, keys: number[][], targetPath: string) {
    return invoke('kv_batch_export', {
        session,
        keys,
        targetPath,
    })
}

export function _kvBatchImport(session: number, targetPath: string, putStrategy: PutStrategy, prefix?: string) {
    return invoke('kv_batch_import', {
        session,
        targetPath,
        putStrategy,
        prefix
    })
}