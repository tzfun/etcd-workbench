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

export type EditorConfig = {
    disabled: boolean
    indentWithTab: boolean
    tabSize: number
    autofocus: boolean
    height: number | 'auto'
    language: string
    theme: string
}