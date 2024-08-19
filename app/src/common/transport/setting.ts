export interface SettingConfig {
    //  应用主题
    theme: 'dark' | 'light' | 'auto',
    //  编辑器黑色主题
    editorDarkTheme: string,
    //  编辑器白色主题
    editorLightTheme: string,
    //  KV路径分割符号，用于树状展示
    kvPathSplitter:string,
    //  KV分页查询
    kvPaginationQuery: true,
    //  KV分页获取每页大小
    kvLimitPerPage: number,
    //  自动下载更新
    autoDownloadUpdate: true,
    //  使用 ctrl + w 关闭连接tab
    closeTabUseCtrlW: true,
    //  连接超时秒数
    connectTimeoutSeconds: number,
    //  请求超时秒数
    requestTimeoutSeconds: number,
    //  窗口初始化状态
    windowInitState?: SettingWindowState,
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
    kvLimitPerPage: 5000,
    closeTabUseCtrlW: true,
    autoDownloadUpdate: true,
    connectTimeoutSeconds: 5,
    requestTimeoutSeconds: 15
}