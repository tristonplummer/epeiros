use crate::io::{Deserialize, GameVersion, ShaiyaReadExt};

/// The prefix that will appear at the beginning of a text file if it is encoded with UTF-16, little endian.
/// https://learn.microsoft.com/en-us/windows/win32/intl/using-byte-order-marks
const UTF16_LE: u16 = 0xFFFE;

/// The prefix that will appear at the beginning of a text file if it is encoded with UTF-16, big endian.
/// https://learn.microsoft.com/en-us/windows/win32/intl/using-byte-order-marks
const UTF16_BE: u16 = 0xFEFF;

impl Deserialize for String {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: std::io::Read + byteorder::ReadBytesExt,
        Self: Sized,
    {
        let bytes = src.consume_all();
        let byte_order_mark = u16::from_be_bytes([bytes[0], bytes[1]]);

        return match byte_order_mark {
            UTF16_BE => Ok(parse_string_utf16_be(&bytes[2..])),
            UTF16_LE => Ok(parse_string_utf16_le(&bytes[2..])),
            _ => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        };
    }
}

/// Parses a String encoded with UTF-16-BE from an input.
///
/// # Arguments
/// * `src` - The source.
fn parse_string_utf16_be(src: &[u8]) -> String {
    let codepoints = src
        .chunks_exact(2)
        .map(|a| u16::from_be_bytes([a[0], a[1]]))
        .collect::<Vec<u16>>();

    String::from_utf16_lossy(&codepoints)
}

/// Parses a String encoded with UTF-16-LE from an input.
///
/// # Arguments
/// * `src` - The source.
fn parse_string_utf16_le(src: &[u8]) -> String {
    let codepoints = src
        .chunks_exact(2)
        .map(|a| u16::from_le_bytes([a[0], a[1]]))
        .collect::<Vec<u16>>();

    String::from_utf16_lossy(&codepoints)
}
