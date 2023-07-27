export interface KeyDTO {
    key: string;
    version: number | 0;
    createRevision: number | 0;
    modRevision: number | 0;
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
    height: string | 'auto'
    language: string
    theme: string,
    fontSize: string,
}

export type TreeNode = {
    path: string,
    type: 'dir' | 'file',
    label: string,
    children?: TreeNode[],
    data?: KeyValueDTO
}