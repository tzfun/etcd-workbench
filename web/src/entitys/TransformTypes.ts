export interface KeyDTO {
    key: string;
    version?: number | 0;
    createVersion?: number | 0;
    modVersion?: number | 0;
    lease?: number | 0;
}

export interface KeyValueDTO extends KeyDTO {
    value: string | undefined;
}