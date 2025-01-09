pub mod file_util;
pub mod aes_util;
pub mod k8s_formatter;
mod test;


pub fn md5(content: impl AsRef<[u8]>) -> String {
    let digest = md5::compute(content);
    format!("{:x}", digest)
}