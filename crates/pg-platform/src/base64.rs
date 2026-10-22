//! Standard base64 (RFC 4648, padded), for save files in the browser's `localStorage`, which
//! only holds text.

#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PADDING: u8 = b'=';

pub fn encode(data: &[u8]) -> String {
    let mut encoded = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let group = (u32::from(first) << 16) | (u32::from(second) << 8) | u32::from(third);

        let symbols = [
            ALPHABET[(group >> 18) as usize & 63],
            ALPHABET[(group >> 12) as usize & 63],
            if chunk.len() > 1 { ALPHABET[(group >> 6) as usize & 63] } else { PADDING },
            if chunk.len() > 2 { ALPHABET[group as usize & 63] } else { PADDING },
        ];
        for symbol in symbols {
            encoded.push(char::from(symbol));
        }
    }
    encoded
}

fn symbol_value(symbol: u8) -> Option<u32> {
    let value = match symbol {
        b'A'..=b'Z' => symbol - b'A',
        b'a'..=b'z' => symbol - b'a' + 26,
        b'0'..=b'9' => symbol - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    };
    Some(u32::from(value))
}

/// The bytes of padded base64 text; `None` when the text is not valid base64.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }

    let mut decoded = Vec::with_capacity(bytes.len() / 4 * 3);
    let group_count = bytes.len() / 4;
    for (index, group) in bytes.chunks(4).enumerate() {
        let is_last = index + 1 == group_count;
        let trailing = group.iter().rev();
        let padding = trailing.take_while(|symbol| **symbol == PADDING).count();
        if padding > 2 || (padding > 0 && !is_last) {
            return None;
        }

        let mut value = 0u32;
        for symbol in &group[..4 - padding] {
            value = (value << 6) | symbol_value(*symbol)?;
        }
        value <<= 6 * padding as u32;

        let group_bytes = [(value >> 16) as u8, (value >> 8) as u8, value as u8];
        decoded.extend_from_slice(&group_bytes[..3 - padding]);
    }
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_4648_vectors() {
        let vectors = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (plain, encoded) in vectors {
            assert_eq!(encode(plain.as_bytes()), encoded);
            assert_eq!(decode(encoded).as_deref(), Some(plain.as_bytes()));
        }
    }

    #[test]
    fn every_byte_round_trips() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode(&data);
        assert_eq!(decode(&encoded), Some(data));
    }

    #[test]
    fn invalid_text_is_rejected() {
        assert_eq!(decode("Zm9"), None);
        assert_eq!(decode("Zm9v!A=="), None);
        assert_eq!(decode("Zg==Zg=="), None);
        assert_eq!(decode("Z==="), None);
    }
}
