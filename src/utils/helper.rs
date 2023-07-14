use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

const BASE62_ALPHABET: [u8; 62] =
    *b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const SCALE: usize = 62;

/// usize convert to base62 string
///
/// # Arguments
///
/// * `data`: usize number
///
/// returns: String
///
/// # Examples
///
/// ```
///  let res = helper::encode_base62(63);
///  assert_eq!(res, "11");
///  let res = helper::encode_base62(1687704559678);
///  assert_eq!(res, "TiCiKz8");
/// ```
#[allow(dead_code)]
pub fn encode_base62(data: usize) -> String {
    if data == 0 {
        return String::from(char::from(BASE62_ALPHABET[data]));
    }
    let mut num = data;
    let mut result = String::with_capacity(8);
    while num > 0 {
        result.insert(0, char::from(BASE62_ALPHABET[num % SCALE]));
        num /= SCALE;
    }
    result
}

/// base62 string convert to usize
///
/// # Arguments
///
/// * `data`: base62 string
///
/// returns: usize
///
/// # Examples
///
/// ```
/// let result = helper::decode_base62("abcd");
/// assert_eq!(result, 8724431);
/// let result = helper::decode_base62("GW");
/// assert_eq!(result, 1024);
/// ```
#[allow(dead_code)]
pub fn decode_base62(data: &str) -> Result<usize, anyhow::Error> {
    let mut result = 0;
    for (i, c) in data.chars().rev().enumerate() {
        let pos = BASE62_ALPHABET.iter().position(|&x| x == c as u8);
        match pos {
            None => {
                anyhow::bail!("invalid short link: [{}]", c)
            }
            Some(index) => {
                let value = index as usize;
                let power = usize::pow(SCALE, i as u32);
                result += value * power;
            }
        }
    }

    Ok(result)
}

/// string convert to sha-256 base64
///
/// # Arguments
///
/// * `input`: not empty string
///
/// returns: base64 string
///
/// # Examples
///
/// ```
/// let result = helper::calculate_sha256("abcd");
/// assert_eq!(result, "iNQmb9TmM40TuEX88olXnSCciXgjuSF9o-Fhk28DFYk");
/// ```
#[allow(dead_code)]
pub fn calculate_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();

    URL_SAFE_NO_PAD.encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha_to_256() {
        let result = calculate_sha256("abcd");
        assert_eq!(result, "iNQmb9TmM40TuEX88olXnSCciXgjuSF9o-Fhk28DFYk");
    }
}
