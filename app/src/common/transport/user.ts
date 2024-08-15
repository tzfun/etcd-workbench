export interface User {
    user: string,
    roles: string[],
    root: boolean
}

export enum RolePermType {
    Read = 0,
    Write = 1,
    ReadAndWrite = 2
}

export interface RolePermission {
    key: string,
    permType: RolePermType,
    prefix: boolean,
    allKeys: boolean
}