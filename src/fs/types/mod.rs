mod item;
mod sdata;
mod skilldata;
mod text;

use crate::io::{Deserialize, GameVersion, Serialize};
use byteorder::{ReadBytesExt, WriteBytesExt};
pub use item::*;
pub use sdata::*;
pub use skilldata::*;
use std::io::{ErrorKind, Read, Write};
pub use text::*;

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum PermittedRace {
    #[default]
    Human,
    Elf,
    AllLight,
    DeathEater,
    Vail,
    AllFury,
    AllFactions,
}

impl Deserialize for PermittedRace {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let permitted = src.read_u8()?;
        return match permitted {
            0 => Ok(PermittedRace::Human),
            1 => Ok(PermittedRace::Elf),
            2 => Ok(PermittedRace::AllLight),
            3 => Ok(PermittedRace::DeathEater),
            4 => Ok(PermittedRace::Vail),
            5 => Ok(PermittedRace::AllFury),
            6 => Ok(PermittedRace::AllFactions),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid permitted race {permitted}"),
            )),
        };
    }
}

impl Serialize for PermittedRace {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            PermittedRace::Human => 0,
            PermittedRace::Elf => 1,
            PermittedRace::AllLight => 2,
            PermittedRace::DeathEater => 3,
            PermittedRace::Vail => 4,
            PermittedRace::AllFury => 5,
            PermittedRace::AllFactions => 6,
        };
        dst.write_u8(id)?;
        Ok(())
    }
}
