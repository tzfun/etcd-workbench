use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub encrypt_key: &'static str,
    pub theme: &'static str
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            encrypt_key: "etcdWorkbench@*?",
            theme: "auto"
        }
    }
}