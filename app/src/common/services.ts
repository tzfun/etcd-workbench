import {invoke} from "@tauri-apps/api";
import {Connection, SessionData} from "~/common/transport/connection.ts";

export function _connectTest(connection: Connection): Promise<undefined> {
    return invoke('connect_test', {connection})
}

export function _connect(connection: Connection): Promise<SessionData> {
    return invoke('connect', {connection: connection})
}