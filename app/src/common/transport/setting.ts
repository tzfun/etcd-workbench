export interface SettingConfig {
    //  应用主题
    theme: 'dark' | 'light' | 'auto',
    //  编辑器黑色主题
    editorDarkTheme: string,
    //  编辑器白色主题
    editorLightTheme: string,
    //  KV路径分割符号，用于树状展示
    kvPathSplitter:string,
    //  KV分页获取每页大小
    kvLimitPerPage: number,
    //  自动下载更新
    autoDownloadUpdate: true,
    //  主窗口初始化宽度
    mainWindowWidth: number | 0,
    //  主窗口初始化高度
    mainWindowHeight: number | 0
}
