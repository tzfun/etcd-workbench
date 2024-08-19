use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct SettingConfig {
    /// 应用主题
    pub theme: String,
    /// 编辑器黑色主题
    pub editor_dark_theme: String,
    /// 编辑器白色主题
    pub editor_light_theme: String,
    /// KV路径分割符号，用于树状展示
    pub kv_path_splitter: String,
    /// KV分页查询
    pub kv_pagination_query: bool,
    /// KV分页获取每页大小
    pub kv_limit_per_page: u32,
    /// 自动下载更新
    pub auto_download_update: bool,
    /// 使用 ctrl + w 关闭连接tab
    pub close_tab_use_ctrl_w: bool,
    /// 连接超时秒数
    pub connect_timeout_seconds: u64,
    /// 请求超时秒数
    pub request_timeout_seconds: u64,
    /// 窗口初始化状态
    pub window_init_state: Option<SettingWindowState>
}

impl Default for SettingConfig {
    fn default() -> Self {
        SettingConfig {
            theme: "auto".to_string(),
            editor_dark_theme: "barf".to_string(),
            editor_light_theme: "smoothy".to_string(),
            kv_path_splitter: "/".to_string(),
            kv_pagination_query: true,
            kv_limit_per_page: 5000,
            auto_download_update: true,
            close_tab_use_ctrl_w: true,
            connect_timeout_seconds: 5,
            request_timeout_seconds: 15,
            window_init_state: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all="camelCase")]
pub struct SettingWindowState {
    /// 主窗口初始化宽度
    pub main_window_width: f64,
    /// 主窗口初始化高度
    pub main_window_height: f64,
    /// 主窗口初始化是否全屏
    pub main_window_fullscreen: bool,
    /// 主窗口初始化是否最大化
    pub main_window_maximize: bool
}