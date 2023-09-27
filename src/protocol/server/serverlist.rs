use crate::io::{Deserialize, Serialize, ShaiyaReadExt, ShaiyaWriteExt};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct ServerList {
    pub servers: Vec<ServerEntry>,
}

#[derive(Debug, Clone)]
pub enum ServerStatus {
    Normal,
    Locked,
    Closed,
}

#[derive(Debug, Clone)]
pub struct ServerEntry {
    pub id: u8,
    pub status: ServerStatus,
    pub player_count: u16,
    pub player_capacity: u16,
    pub name: String,
}

impl ServerStatus {
    /// Gets the byte representation of a given server status.
    fn id(&self) -> u8 {
        match *self {
            ServerStatus::Normal => 0,
            ServerStatus::Locked => 1,
            ServerStatus::Closed => 2,
        }
    }

    /// Maps a `u8` to a [ServerStatus]. If none is found, this
    /// returns an Error.
    ///
    /// # Arguments
    /// * `id`  - The status id.
    fn for_id(id: u8) -> Option<ServerStatus> {
        match id {
            0 => Some(ServerStatus::Normal),
            1 => Some(ServerStatus::Locked),
            2 => Some(ServerStatus::Closed),
            _ => None,
        }
    }
}

impl Serialize for ServerList {
    type Error = std::io::Error;

    fn serialize<T>(&self, dst: &mut T) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_u8(self.servers.len() as u8)?;
        self.servers.iter().try_for_each(|e| e.serialize(dst))?;
        Ok(())
    }
}

impl Deserialize for ServerList {
    type Error = std::io::Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let qty = src.read_u8()?;
        let mut servers = Vec::with_capacity(qty as usize);
        for _ in 0..qty {
            servers.push(ServerEntry::deserialize(src)?);
        }

        Ok(Self { servers })
    }
}

impl Serialize for ServerEntry {
    type Error = std::io::Error;

    fn serialize<T>(&self, dst: &mut T) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_u8(self.id)?;
        dst.write_u8(self.status.id())?;
        dst.write_u16::<LittleEndian>(self.player_count)?;
        dst.write_u16::<LittleEndian>(self.player_capacity)?;
        dst.write_string(&self.name, 32)?;
        Ok(())
    }
}

impl Deserialize for ServerEntry {
    type Error = std::io::Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let id = src.read_u8()?;
        let status = match ServerStatus::for_id(src.read_u8()?) {
            Some(status) => status,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "invalid server status",
                ))
            }
        };

        let player_count = src.read_u16::<LittleEndian>()?;
        let player_capacity = src.read_u16::<LittleEndian>()?;
        let name = src.read_string(32)?;

        Ok(Self {
            id,
            status,
            player_count,
            player_capacity,
            name,
        })
    }
}
