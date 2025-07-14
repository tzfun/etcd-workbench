import {EditorHighlightLanguage} from "~/common/types.ts";

export const MAIN_WINDOW_MIN_HEIGHT = 500;
export const MAIN_WINDOW_MIN_WIDTH = 900;
export type UpdateSource = 'github' | 'gitee';

export interface SettingConfig {
    //  应用主题
    theme: 'dark' | 'light' | 'auto',
    //  编辑器黑色主题
    editorDarkTheme: string,
    //  编辑器白色主题
    editorLightTheme: string,

    //  KV路径分割符号，用于树状展示
    kvPathSplitter: string,
    //  KV分页查询
    kvPaginationQuery: boolean,
    //  KV分页获取每页大小
    kvLimitPerPage: number | string,
    //  KV保存之前是否检查格式
    kvCheckFormatBeforeSave: boolean,
    kvTreeSearchWithFolder: boolean,

    //  自动下载更新
    autoUpdate: boolean,
    //  更新源
    updateSource: UpdateSource,

    //  使用 ctrl + w 关闭连接tab
    closeTabUseCtrlW: boolean,
    //  连接超时秒数
    connectTimeoutSeconds: number | string,
    //  请求超时秒数
    requestTimeoutSeconds: number | string,
    //  SSH连接超时秒数
    sshConnectTimeoutSeconds: number | string,
    //  连接存储加密密钥，bytes字符长度必须为16位
    connectionConfEncryptKey: string,
}

export interface SettingWindowState {
    mainWindowWidth: number,
    mainWindowHeight: number,
    mainWindowFullscreen: boolean,
    mainWindowMaximize: boolean
}

export const DEFAULT_SETTING_CONFIG: SettingConfig = {
    theme: 'auto',
    editorDarkTheme: 'barf',
    editorLightTheme: 'smoothy',
    kvPathSplitter: '/',
    kvPaginationQuery: true,
    kvLimitPerPage: 2000,
    kvCheckFormatBeforeSave: true,
    kvTreeSearchWithFolder: true,
    closeTabUseCtrlW: true,
    autoUpdate: true,
    updateSource: 'github',
    connectTimeoutSeconds: 5,
    requestTimeoutSeconds: 15,
    sshConnectTimeoutSeconds: 10,
    connectionConfEncryptKey: 'workbench*#)&%.$'
}

export interface FileFormat {
    //  key全路径，包含namespace前缀
    key: string,
    format: EditorHighlightLanguage
}

export interface GlobalStoreConfig {
    //  窗口初始化状态
    windowInitState?: SettingWindowState,
    fileFormatLog: FileFormat[],
    fileFormatLogMap: Record<string, EditorHighlightLanguage>
}

export const DEFAULT_GLOBAL_STORE: GlobalStoreConfig = {
    fileFormatLog: [],
    fileFormatLogMap: {}
}