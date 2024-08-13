export interface KeyValue {
    key: string,
    createRevision: number,
    modRevision: number,
    version: number,
    value: number[],
    lease: string,
    leaseInfo?: LeaseSimpleInfo
}

export interface LeaseInfo {
    id: string,
    ttl: number,
    grantedTtl: number,
    keys: string[]
}

export interface LeaseSimpleInfo {
    ttl: number,
    grantedTtl: number,
}