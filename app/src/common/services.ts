import {invoke} from "@tauri-apps/api";
import {Connection, ConnectionInfo, SessionData} from "~/common/transport/connection.ts";
import {Cluster} from "~/common/transport/maintenance.ts";

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