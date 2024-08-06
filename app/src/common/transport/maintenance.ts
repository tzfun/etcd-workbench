export interface Cluster {
    id: string,
    revision: number,
    raftTerm: string,
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
    dbSize: number,
    raftUsedDbSize: number,
    leader: string,
    raftIndex: string,
    raftTerm: string,
    raftAppliedIndex: string,
    errors: string[]
}