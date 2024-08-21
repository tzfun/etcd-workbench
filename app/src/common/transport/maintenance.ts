export enum Alarm {
    None = 0,
    Nospace = 1,
    Corrupt = 2,
}

export interface Cluster {
    id: string,
    memberId: string,
    revision: number,
    members: ClusterMember[],
    status: ClusterStatus
}

export interface ClusterMember {
    id: string,
    name: string,
    peerUri: string[],
    clientUri: string[],
    alarmType: number
}

export interface ClusterStatus {
    version: string,
    dbSizeAllocated: number,
    dbSizeUsed: number,
    leader: string,
    raftIndex: string,
    raftTerm: string,
    raftAppliedIndex: string,
    errors: string[]
}

export interface SnapshotState {
    received: number,
    remain: number,
    errorMsg?: string,
    finished: boolean
}

export interface SnapshotInfo {
    name: string,
    folder: string,
    id: number,
    state: SnapshotState
}

export interface SnapshotStateEvent {
    id: number,
    state: SnapshotState
}