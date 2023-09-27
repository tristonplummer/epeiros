use crate::io::{Deserialize, GameVersion, ShaiyaReadExt};

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

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let mut data = src.consume_all();
        if !is_encrypted(&data) {
            return Ok(Self { data });
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

pub(crate) fn ep6_or_above(version: GameVersion) -> bool {
    version >= GameVersion::Ep6
}

pub(crate) fn ep6v2_or_above(version: GameVersion) -> bool {
    version >= GameVersion::Ep6v2
}

macro_rules! user_type {
    ($typ:ty) => {
        $typ
    };
}

macro_rules! user_type_readable {
    ($src:ident, $version:ident, u8, $if:expr) => {
        if $if($version) {
            $src.read_u8()?
        } else {
            0
        }
    };
    ($src:ident, $version:ident, u8) => {
        $src.read_u8()?
    };
    ($src:ident, $version:ident, bool) => {
        1 >= $src.read_u8()?
    };
    ($src:ident, $version:ident, bool, $if:expr) => {
        if $if($version) {
            1 >= $src.read_u8()?
        } else {
            false
        }
    };
    ($src:ident, $version:ident, u16) => {
        $src.read_u16::<byteorder::LittleEndian>()?
    };
    ($src:ident, $version:ident, u16, $if:expr) => {
        if $if($version) {
            $src.read_u16::<byteorder::LittleEndian>()?
        } else {
            0
        }
    };
    ($src:ident, $version:ident, u32) => {
        $src.read_u32::<byteorder::LittleEndian>()?
    };
    ($src:ident, $version:ident, u32, $if:expr) => {
        if $if($version) {
            $src.read_u32::<byteorder::LittleEndian>()?
        } else {
            0
        }
    };
    ($src:ident, $version:ident, String) => {
        $src.read_length_prefixed_string()?
    };
    ($src:ident, $version:ident, String, $if:expr) => {
        if $if($version) {
            $src.read_length_prefixed_string()?
        } else {
            String::new()
        }
    };
    ($src:ident, $version:ident, Vec <$inner:ident>, $len:expr) => {{
        let length = $len($version);
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            let inner = $inner::versioned_deserialize($src, $version)?;
            vec.push(inner);
        }
        vec
    }};
    ($src:ident, $version:ident, $typ:ty) => {
        <$typ>::versioned_deserialize($src, $version)?
    };
    ($src:ident, $version:ident, $typ:ty, $if:expr) => {
        if $if($version) {
            <$typ>::versioned_deserialize($src, $version)?
        } else {
            <$typ>::default()
        }
    };
}

macro_rules! user_type_writeable {
    ($dst:ident, $version:ident, u8, $value:expr, $if:expr) => {
        if $if($version) {
            $dst.write_u8(*$value)?
        }
    };
    ($dst:ident, $version:ident, u8, $value:expr) => {
        $dst.write_u8(*$value)?
    };
    ($dst:ident, $version:ident, bool, $value:expr) => {
        $dst.write_u8(if *$value { 1 } else { 0 })?
    };
    ($dst:ident, $version:ident, bool, $value:expr, $if:expr) => {
        if $if($version) {
            $dst.write_u8(if *$value { 1 } else { 0 })?
        }
    };
    ($dst:ident, $version:ident, u16, $value:expr) => {
        $dst.write_u16::<byteorder::LittleEndian>(*$value)?
    };
    ($dst:ident, $version:ident, u16, $value:expr, $if:expr) => {
        if $if($version) {
            $dst.write_u16::<byteorder::LittleEndian>(*$value)?
        }
    };
    ($dst:ident, $version:ident, u32, $value:expr) => {
        $dst.write_u32::<byteorder::LittleEndian>(*$value)?
    };
    ($dst:ident, $version:ident, u32, $value:expr, $if:expr) => {
        if $if($version) {
            $dst.write_u32::<byteorder::LittleEndian>(*$value)?
        }
    };
    ($dst:ident, $version:ident, String, $value:expr) => {
        $dst.write_length_prefixed_string($value)?
    };
    ($dst:ident, $version:ident, String, $value:expr, $if:expr) => {
        if $if($version) {
            $dst.write_length_prefixed_string($value)?
        }
    };
    ($dst:ident, $version:ident, Vec <$inner:ident>, $value:expr, $len:expr) => {{
        let length = $len($version);
        for idx in 0..length {
            if idx >= $value.len() {
                let default = $inner::default();
                default.versioned_serialize($dst, $version)?;
            } else {
                $value[idx].versioned_serialize($dst, $version)?;
            }
        }
    }};
    ($dst:ident, $version:ident, $typ:ty, $value:expr) => {
        $value.versioned_serialize($dst, $version)?
    };
    ($dst:ident, $version:ident, $typ:ty, $value:expr, $if:expr) => {
        if $if($version) {
            $value.versioned_serialize($dst, $version)?
        }
    };
}

macro_rules! sdata_record {
    (
        $ident:ident {
            $(
                $field:ident $typ:ident $(<$generics:ident>)?
                $(if($if:expr))?
                $(len($len:expr))?
            );* $(;)?
        }
    ) => {
        #[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
        pub struct $ident {
            $(
                #[serde(skip_serializing_if = "crate::fs::types::sdata::is_default")]
                #[serde(default)]
                pub $field: user_type!($typ $(<$generics>)?),
            )*
        }

        impl $crate::io::Deserialize for $ident {
            type Error = std::io::Error;

            #[allow(unused_variables)]
            fn versioned_deserialize<T>(src: &mut T, version: GameVersion) -> Result<Self, Self::Error>
            where
                T: Read + ReadBytesExt,
                Self: Sized
            {
                $(
                    let $field = user_type_readable!(src, version, $typ $(<$generics>)? $(,$if)? $(,$len)?);
                )*

                Ok(Self {
                    $(
                        $field,
                    )*
                })
            }
        }

        impl $crate::io::Serialize for $ident {
            type Error = std::io::Error;

            #[allow(unused_variables)]
            fn versioned_serialize<T>(&self, dst: &mut T, version: GameVersion) -> Result<(), Self::Error>
            where
                T: Write + WriteBytesExt
            {
                $(
                    user_type_writeable!(dst, version, $typ $(<$generics>)?, &self.$field $(,$if)? $(,$len)?);
                )*
                Ok(())
            }
        }
    };
}

pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub(crate) use {sdata_record, user_type, user_type_readable, user_type_writeable};
