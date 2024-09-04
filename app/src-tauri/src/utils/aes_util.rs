#![allow(unused)]
use aes::cipher::consts::U16;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use std::cmp::min;
use std::fmt::Display;

pub const LENGTH_16: usize = 16;

#[derive(Debug)]
pub enum AesError {
    InvalidKeyLength(String),
    InvalidBlockLength,
    TryFromSliceError
}

impl Display for AesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AesError::InvalidKeyLength(s) => {
                write!(f, "aes crypt error: InvalidKeyLength {}", s)
            },
            AesError::InvalidBlockLength => {
                write!(f, "aes crypt error: InvalidBlockLength")
            },
            AesError::TryFromSliceError => {
                write!(f, "aes crypt error: TryFromSliceError")
            }
        }
    }
}

pub fn encrypt_128(key: &[u8], content: impl Into<Vec<u8>>) -> Result<Vec<u8>, AesError> {
    let key_len = key.len();
    if key_len != LENGTH_16 {
        return Err(AesError::InvalidKeyLength(format!("Invalid aes key length: {}, expect: {}", key_len, LENGTH_16)));
    }
    let k = GenericArray::from_slice(key);
    let cipher = Aes128::new(&k);

    let blocks = &mut encode_aes_block_content_16(content)?[..];
    cipher.encrypt_blocks(blocks);
    Ok(merge_blocks(blocks))
}

pub fn decrypt_128(key: &[u8], content: impl Into<Vec<u8>>) -> Result<Vec<u8>, AesError> {
    let key_len = key.len();
    if key_len != LENGTH_16 {
        return Err(AesError::InvalidKeyLength(format!("Invalid aes key length: {}, expect: {}", key_len, LENGTH_16)));
    }
    let k = GenericArray::from_slice(key);
    let cipher = Aes128::new(&k);

    let mut blocks = vec![];
    let content_vec = content.into();
    let len = content_vec.len();
    let mut idx = 0;
    while idx < len {
        let end_idx = idx + LENGTH_16;
        if end_idx > len {
            return Err(AesError::InvalidBlockLength);
        }
        let array: [u8; LENGTH_16] = content_vec[idx..end_idx].try_into().map_err(|_| AesError::TryFromSliceError)?;
        blocks.push(GenericArray::from(array));
        idx += LENGTH_16;
    }

    let blocks = &mut blocks[..];

    cipher.decrypt_blocks(blocks);

    decode_aes_block_content_16(&blocks)
}

fn encode_aes_block_content_16(content: impl Into<Vec<u8>>) -> Result<Vec<GenericArray<u8, U16>>, AesError> {
    let content_vec = content.into();
    let mut blocks = vec![];

    let mut idx = 0;
    let len = content_vec.len();
    let mut filled = false;
    while idx < len {
        let split_size = min(len - idx, LENGTH_16);
        if split_size == LENGTH_16 {
            let split_idx = idx + split_size;
            let array: [u8; LENGTH_16] = content_vec[idx..split_idx].try_into().map_err(|_| AesError::TryFromSliceError)?;
            blocks.push(GenericArray::from(array));
            idx += LENGTH_16;
        } else {
            let mut array = [0u8; LENGTH_16];

            for i in 0..split_size {
                array[i] = content_vec[i + idx];
            }
            filled = true;
            if split_size == LENGTH_16 - 1 {
                //  只剩1位，补全一个block
                array[LENGTH_16 - 1] = LENGTH_16 as u8;
                blocks.push(GenericArray::from(array));
                blocks.push(GenericArray::from([0u8; LENGTH_16]));
            } else {
                //  剩余位数进行补位，格式为 [x,x,x,n,0,0,0...]
                //  其中x为真实数据， n为补位标志位记录后面0的数量，0为补位值
                array[split_size] = (LENGTH_16 - split_size - 1) as u8;
                blocks.push(GenericArray::from(array));
            }

            break;
        }
    }

    //  如果没有补位，需对内容进行检测，防止真实内容与补位格式恰好重叠
    if !filled {
        let blocks_idx = blocks.len() - 1;
        let mut zero_count = 0;
        let mut refill = false;
        'block_loop: for idx in (0..blocks_idx).rev() {
            let block = blocks[idx];
            for i in (0..LENGTH_16).rev() {
                let v = block[i];
                if v == 0 {
                    zero_count += 1;
                } else {
                    if zero_count == v {
                        refill = true;
                    } else {
                        break 'block_loop;
                    }
                }
            }
        }

        if refill {
            let mut array = [0u8; LENGTH_16];
            array[0] = (LENGTH_16 - 1) as u8;
            blocks.push(GenericArray::from(array));
        }
    }

    Ok(blocks)
}

fn decode_aes_block_content_16(blocks: &[GenericArray<u8, U16>]) -> Result<Vec<u8>, AesError> {
    let mut content = vec![];

    let blocks_len = blocks.len();
    let mut zero_count = 0;
    let mut filled = false;
    let mut filled_idx = (0usize, 0usize);
    'block_loop: for idx in (0..blocks_len).rev() {
        let block = blocks[idx];
        if block.len() != LENGTH_16 {
            return Err(AesError::InvalidBlockLength);
        }
        for i in (0..LENGTH_16).rev() {
            let v = block[i];
            if v == 0 {
                zero_count += 1;
            } else {
                if zero_count == v {
                    filled = true;
                    filled_idx.0 = idx;
                    filled_idx.1 = i;
                }
                break 'block_loop;
            }
        }
    }

    'block_loop: for idx in 0..blocks_len {
        let block = blocks[idx];
        for i in 0..LENGTH_16 {
            if filled && idx == filled_idx.0 && i == filled_idx.1 {
                break 'block_loop;
            }
            content.push(block[i]);
        }
    }

    Ok(content)
}

fn merge_blocks(blocks: &[GenericArray<u8, U16>]) -> Vec<u8> {
    let mut content = vec![];
    let blocks_len = blocks.len();
    for idx in 0..blocks_len {
        let block = blocks[idx];
        let block_len = block.len();
        for i in 0..block_len {
            content.push(block[i]);
        }
    }
    content
}
