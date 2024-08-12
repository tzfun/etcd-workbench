export interface KeyValue {
    key: string,
    createRevision: number,
    modRevision: number,
    version: number,
    value: number[],
    lease: string
}