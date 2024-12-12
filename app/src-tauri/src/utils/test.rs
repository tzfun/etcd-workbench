#![cfg(test)]
use super::aes_util;

const KEY: &'static str = "1234567890123!@#";

#[test]
fn test_aes() {
    let content = "h";    //  长度1
    let encrypted = aes_util::encrypt_128(KEY.as_bytes(), content).unwrap();
    let decrypted = aes_util::decrypt_128(KEY.as_bytes(), encrypted).unwrap();
    let res = String::from_utf8(decrypted).unwrap();
    assert_eq!(content, res);

    let content = "123456789012345";    //  长度15
    let encrypted = aes_util::encrypt_128(KEY.as_bytes(), content).unwrap();
    let decrypted = aes_util::decrypt_128(KEY.as_bytes(), encrypted).unwrap();
    let res = String::from_utf8(decrypted).unwrap();
    assert_eq!(content, res);

    let content = "hello!!你好啊";  //  长度16
    let encrypted = aes_util::encrypt_128(KEY.as_bytes(), content).unwrap();
    let decrypted = aes_util::decrypt_128(KEY.as_bytes(), encrypted).unwrap();
    let res = String::from_utf8(decrypted).unwrap();
    assert_eq!(content, res);
    
    let content = "hello!!你好啊a"; //  长度 17
    let encrypted = aes_util::encrypt_128(KEY.as_bytes(), content).unwrap();
    let decrypted = aes_util::decrypt_128(KEY.as_bytes(), encrypted).unwrap();
    let res = String::from_utf8(decrypted).unwrap();
    assert_eq!(content, res);
}

#[test]
fn test_aes_long_content() {
    let content = "eyJuYW1lIjoiQzFfT25saW5lIiwiY29ubmVjdGlvbiI6eyJob3N0IjoiMTAuMC4wLjE3IiwicG9ydCI6MjMyMywibmFtZXNwYWNlIjpudWxsLCJ1c2VyIjp7InVzZXJuYW1lIjoicm9vdCIsInBhc3N3b3JkIjoiNlBUdWtBOEdWMnZYeFRrVHhxNXcifSwidGxzIjpudWxsLCJ";
    let encrypted = aes_util::encrypt_128(KEY.as_bytes(), content).unwrap();
    let decrypted = aes_util::decrypt_128(KEY.as_bytes(), encrypted).unwrap();
    let res = String::from_utf8(decrypted).unwrap();
    assert_eq!(content, res);
}

#[test]
fn test_aes_json_content() {
    let content = "{\"name\":\"localhost\",\"connection\":{\"host\":\"127.0.0.1\",\"port\":2379,\"namespace\":null,\"user\":null,\"tls\":null,\"ssh\":null},\"keyCollection\":[\"/tz_mac/config/battle/config-server.json\"],\"keyMonitorList\":[]}";
    let encrypted = aes_util::encrypt_128(KEY.as_bytes(), content).unwrap();
    let decrypted = aes_util::decrypt_128(KEY.as_bytes(), encrypted).unwrap();
    let res = String::from_utf8(decrypted).unwrap();
    assert_eq!(content, res);
}