use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct SettingConfig {
    /// 应用主题
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 编辑器黑色主题
    #[serde(default = "default_editor_dark_theme")]
    pub editor_dark_theme: String,
    /// 编辑器白色主题
    #[serde(default = "default_editor_light_theme")]
    pub editor_light_theme: String,

    /// KV路径分割符号，用于树状展示
    #[serde(default = "default_kv_path_splitter")]
    pub kv_path_splitter: String,
    /// KV分页查询
    #[serde(default)]
    pub kv_pagination_query: bool,
    /// KV分页获取每页大小
    #[serde(default = "default_kv_limit_per_page")]
    pub kv_limit_per_page: u32,
    /// KV保存之前是否检查格式
    #[serde(default = "default_kv_check_format_before_save")]
    pub kv_check_format_before_save: bool,

    /// 自动更新
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,

    /// 使用 ctrl + w 关闭连接tab
    #[serde(default)]
    pub close_tab_use_ctrl_w: bool,
    /// 连接超时秒数
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
    /// 请求超时秒数
    #[serde(default = "default_request_timeout_seconds")]
    pub request_timeout_seconds: u64,
    /// SSH连接超时秒数
    #[serde(default = "default_ssh_connect_timeout_seconds")]
    pub ssh_connect_timeout_seconds: u64,
    /// 连接存储加密密钥，bytes字符长度必须为16位
    #[serde(default = "default_connection_conf_encrypt_key")]
    pub connection_conf_encrypt_key: String,
}

fn default_theme() -> String {
    String::from("auto")
}

fn default_editor_dark_theme() -> String {
    String::from("barf")
}

fn default_editor_light_theme() -> String {
    String::from("smoothy")
}

fn default_kv_path_splitter() -> String {
    String::from("/")
}

fn default_kv_limit_per_page() -> u32 {
    2000
}

fn default_kv_check_format_before_save() -> bool {
    true
}

fn default_connect_timeout_seconds() -> u64 {
    5
}

fn default_request_timeout_seconds() -> u64 {
    15
}

fn default_ssh_connect_timeout_seconds() -> u64 {
    10
}

fn default_auto_update() -> bool {
    true
}

fn default_connection_conf_encrypt_key() -> String {
    String::from("workbench*#)&%.$")
}

impl Default for SettingConfig {
    fn default() -> Self {
        SettingConfig {
            theme: default_theme(),
            editor_dark_theme: default_editor_dark_theme(),
            editor_light_theme: default_editor_light_theme(),
            kv_path_splitter: default_kv_path_splitter(),
            kv_pagination_query: true,
            kv_limit_per_page: default_kv_limit_per_page(),
            kv_check_format_before_save: true,
            auto_update: true,
            close_tab_use_ctrl_w: true,
            connect_timeout_seconds: default_connect_timeout_seconds(),
            request_timeout_seconds: default_request_timeout_seconds(),
            ssh_connect_timeout_seconds: default_ssh_connect_timeout_seconds(),
            connection_conf_encrypt_key: default_connection_conf_encrypt_key(),
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all="camelCase")]
pub struct GlobalStoreConfig {
    /// 窗口初始化状态
    #[serde(default)]
    pub window_init_state: Option<SettingWindowState>,
    //  value格式化类型记录
    pub file_format_log: VecDeque<FileFormat>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FileFormat {
    pub key: String,
    pub format: String
}