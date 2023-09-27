use crate::io::{Deserialize, ShaiyaReadExt};

use byteorder::ReadBytesExt;
use cipher::BlockDecrypt;
use kisaseed::{Block, Key, SEED};
use std::io::Read;

const SEED_SIGNATURE: &str = "0001CBCEBC5B2784D3FC9A2A9DB84D1C3FEB6E99";

const SHAIYA_SEED_KEY: &[u32] = &[
    0x79F5DBDE, 0x345AC74A, 0x0F482438, 0x0131F493, 0x81A8500C, 0x0659BDCF, 0x26FF71C1, 0x86E9A5CB,
    0xCA6FB745, 0x50E2C1AE, 0x381DDAE1, 0xC3402821, 0x3FECCB4A, 0x3E0BE066, 0x372582FF, 0x826317E3,
    0xA47B5369, 0xC9093C0E, 0xE16C9CB1, 0x2E27228E, 0x84E2D1CD, 0xC840F818, 0x44AEA6F8, 0xD298548D,
    0x2040CA27, 0x4B4E2B78, 0x64F0A045, 0xC171A8DA, 0x384855ED, 0xC033578B, 0x2A8703C7, 0xF15DA3A7,
];

#[derive(Debug)]
pub struct SData {
    pub data: Vec<u8>,
}

impl Deserialize for SData {
    type Error = std::io::Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let mut data = src.consume_all();
        if !is_encrypted(&data) {
            return Ok(Self {
                data: data.to_vec(),
            });
        }

        let _checksum = u32::from_le_bytes(data[40..44].try_into().unwrap());
        let decrypted_size = u32::from_le_bytes(data[44..48].try_into().unwrap()) as usize;

        let encrypted = &mut data[64..];
        let seed = SEED::with_key(*Key::from_slice(SHAIYA_SEED_KEY));

        let mut blocks = encrypted
            .chunks_mut(16)
            .map(|chunk| Block::from(unsafe { *(chunk.as_mut_ptr() as *const [u8; 16]) }))
            .collect::<Vec<_>>();
        seed.decrypt_blocks(&mut blocks);

        Ok(Self {
            data: blocks
                .iter()
                .fold(Vec::with_capacity(decrypted_size), |mut acc, e| {
                    acc.extend_from_slice(e.as_slice());
                    acc
                }),
        })
    }
}

fn is_encrypted(buf: &[u8]) -> bool {
    if buf.len() < SEED_SIGNATURE.len() {
        return false;
    }

    String::from_utf8_lossy(&buf[..SEED_SIGNATURE.len()]).eq(SEED_SIGNATURE)
}
