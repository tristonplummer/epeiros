use crate::io::{Deserialize, GameVersion, Serialize, ShaiyaReadExt, ShaiyaWriteExt};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl Serialize for LoginRequest {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_string(&self.username, 32)?;
        dst.write_string(&self.password, 19)?;
        Ok(())
    }
}

impl Deserialize for LoginRequest {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let username = src.read_string(32)?;
        let password = src.read_string(19)?;
        Ok(Self { username, password })
    }
}
