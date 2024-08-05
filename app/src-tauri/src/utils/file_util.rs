use std::{fs, io};
use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

static BASE_DIR: &'static str = "Etcd Workbench";
pub static CONFIG_DIR: &'static str = "configuration";
pub static SETTINGS_FILE: &'static str = "settings";

/// 创建一个临时文件，并返回该文件的全路径
pub fn create_temp_file(data: &[u8]) -> io::Result<String> {
    let mut dir = temp_dir();
    let file_name = format!("{}", Uuid::new_v4());
    dir.push(file_name);
    let file_full_name = dir.display().to_string();
    let mut file = File::create(dir)?;
    file.write(data)?;
    Ok(file_full_name)
}

pub fn init() -> io::Result<()> {
    let dir = get_storage_dir()?;
    let path = Path::new(&dir);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn get_storage_dir() -> io::Result<String> {
    Ok(format!("C:\\ProgramData\\{}", BASE_DIR))
}

#[cfg(unix)]
pub fn get_storage_dir() -> io::Result<String> {
    Ok(format!("~/{}", BASE_DIR))
}