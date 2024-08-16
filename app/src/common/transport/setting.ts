export interface SettingConfig {
    theme: 'dark' | 'light' | 'auto',
    editorTheme: {
        dark: string,
        light: string
    },
    update: {
        autoDownload: true
    },
    size: {
        width: number | 'default'
        height: number | 'default'
    }
}