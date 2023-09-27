use crate::io::{Deserialize, GameVersion, Serialize};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub enum LoginResponse {
    Success {
        user_id: u32,
        privilege: u8,
        identity: u128,
    },
    Fail(LoginErrorCode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginErrorCode {
    AccountDoesntExist = 1,
    CannotConnect = 2,
    InvalidCredentials = 3,
    AccountDisabled = 10,
}

impl Serialize for LoginResponse {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        match self {
            LoginResponse::Success {
                user_id,
                privilege,
                identity,
            } => {
                dst.write_u8(0)?; // success code
                dst.write_u32::<byteorder::LittleEndian>(*user_id)?;
                dst.write_u8(*privilege)?;
                dst.write_u128::<byteorder::LittleEndian>(*identity)?;
            }
            LoginResponse::Fail(error) => {
                dst.write_u8(error.clone() as u8)?;
            }
        }

        Ok(())
    }
}

impl Deserialize for LoginResponse {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let status = src.read_u8()?;
        match status {
            0 => {
                let user_id = src.read_u32::<byteorder::LittleEndian>()?;
                let privilege = src.read_u8()?;
                let identity = src.read_u128::<byteorder::LittleEndian>()?;
                Ok(Self::Success {
                    user_id,
                    privilege,
                    identity,
                })
            }
            1 => Ok(Self::Fail(LoginErrorCode::AccountDoesntExist)),
            3 => Ok(Self::Fail(LoginErrorCode::InvalidCredentials)),
            10 => Ok(Self::Fail(LoginErrorCode::AccountDisabled)),
            _ => Ok(Self::Fail(LoginErrorCode::CannotConnect)),
        }
    }
}
