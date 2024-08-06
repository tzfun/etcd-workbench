import {SessionData} from "~/common/transport/connection.ts";

const sessions: { number: SessionData } = {}

export function _addSession(sessionData: SessionData) {
    sessions[sessionData.id] = sessionData
}

export function _getSession(session: number): SessionData {
    return sessions[session]
}

export function _removeSession(session:number) {
    delete sessions[session]
}