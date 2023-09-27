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
pub enum GameMode {
    #[default]
    Easy,
    Normal,
    Hard,
    Ultimate,
}

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum PermittedRace {
    Human,
    Elf,
    AllLight,
    DeathEater,
    Vail,
    AllFury,
    AllFactions,
    #[default]
    None,
}

#[derive(Default, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub enum ElementType {
    #[default]
    None,
    Fire(u8),
    Water(u8),
    Earth(u8),
    Wind(u8),
}

impl Deserialize for GameMode {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let mode = src.read_u8()?;
        match mode {
            0 => Ok(GameMode::Easy),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Hard),
            3 => Ok(Self::Ultimate),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid game mode {mode}"),
            )),
        }
    }
}

impl Serialize for GameMode {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::Easy => 0,
            Self::Normal => 1,
            Self::Hard => 2,
            Self::Ultimate => 3,
        };
        dst.write_u8(id)
    }
}

impl Deserialize for PermittedRace {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let permitted = src.read_u8()?;
        match permitted {
            0 => Ok(Self::Human),
            1 => Ok(Self::Elf),
            2 => Ok(Self::AllLight),
            3 => Ok(Self::DeathEater),
            4 => Ok(Self::Vail),
            5 => Ok(Self::AllFury),
            6 => Ok(Self::AllFactions),
            7 => Ok(Self::None),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid permitted race {permitted}"),
            )),
        }
    }
}

impl Serialize for PermittedRace {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::Human => 0,
            Self::Elf => 1,
            Self::AllLight => 2,
            Self::DeathEater => 3,
            Self::Vail => 4,
            Self::AllFury => 5,
            Self::AllFactions => 6,
            Self::None => 7,
        };
        dst.write_u8(id)
    }
}

impl Deserialize for ElementType {
    type Error = std::io::Error;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let element_type = src.read_u8()?;
        let element_id = element_type % 4;
        let element_level = element_type / 4 + 1;

        match element_id {
            0 => Ok(Self::None),
            1 => Ok(Self::Fire(element_level)),
            2 => Ok(Self::Water(element_level)),
            3 => Ok(Self::Earth(element_level)),
            4 => Ok(Self::Wind(element_level)),
            _ => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid element type {element_type}"),
            )),
        }
    }
}

impl Serialize for ElementType {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let id = match *self {
            Self::None => 0,
            Self::Fire(level) => ((level - 1) * 4) + 1,
            Self::Water(level) => ((level - 1) * 4) + 2,
            Self::Earth(level) => ((level - 1) * 4) + 3,
            Self::Wind(level) => ((level - 1) * 4) + 4,
        };
        dst.write_u8(id)
    }
}
