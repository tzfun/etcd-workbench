pub mod aes_util;
pub mod file_util;
pub mod k8s_formatter;
mod test;

pub fn md5(content: impl AsRef<[u8]>) -> String {
    let digest = md5::compute(content);
    format!("{:x}", digest)
}

pub fn vec_to_hex<V: AsRef<[u8]>>(v: V) -> String {
    v.as_ref().iter().map(|b| format!("{:02X}", b)).collect()
}

pub fn hex_to_vec<S: AsRef<str>>(hex_string: S) -> Result<Vec<u8>, String> {
    let hex_string = hex_string.as_ref();
    if hex_string.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    
    hex_string
        .as_bytes()
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| {
            let hex_pair = std::str::from_utf8(chunk)
                .map_err(|_| "Invalid UTF-8 in hex string".to_string())?;
            
            u8::from_str_radix(hex_pair, 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i * 2, e))
        })
        .collect()
}
