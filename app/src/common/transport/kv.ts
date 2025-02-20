export enum FormatSource {
    Kubernetes = "kubernetes",
}

export enum FormatLanguage {
    Json = "json",
}

export interface FormattedValue {
    source: FormatSource,
    language: FormatLanguage,
    value: string
}

export interface KeyValue {
    key: string,
    createRevision: number,
    modRevision: number,
    version: number,
    value: number[],
    lease: string,
    leaseInfo?: LeaseSimpleInfo,
    formattedValue?: FormattedValue
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

export interface SearchResult {
    count: number,
    results: KeyValue[]
}

export interface KVPutResult {
    success: boolean,
    finalKv?: KeyValue,
    existValue?: number[],
    existVersion?: number,
}