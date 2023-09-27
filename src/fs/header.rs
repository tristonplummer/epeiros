use crate::io::{Deserialize, ShaiyaReadExt};
use byteorder::ReadBytesExt;
use std::collections::VecDeque;
use std::io::{Cursor, Read};
use std::path::Path;
use thiserror::Error;

const SAH_MAGIC_VALUE: &str = "SAH";

pub struct Header {
    root: VirtualDirectory,
}

pub struct VirtualDirectory {
    name: String,
    subdirectories: Vec<VirtualDirectory>,
    nodes: Vec<Inode>,
}

pub struct Inode {
    pub name: String,
    pub offset: usize,
    pub length: usize,
    pub checksum: u32,
}

#[derive(Error, Debug)]
pub enum HeaderDeserializeError {
    #[error("read error")]
    ReadError(#[from] std::io::Error),

    #[error("invalid magic value (expected {expected:?}, found {found:?})")]
    InvalidMagicValue { expected: String, found: String },
}

impl Header {
    pub fn open<P>(path: P) -> Result<Self, HeaderDeserializeError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let buf = std::fs::read(path)?;

        let mut src = Cursor::new(buf.as_slice());
        Self::deserialize(&mut src)
    }

    pub fn get_inode<T>(&self, virtual_path: T) -> Option<&Inode>
    where
        T: AsRef<str>,
    {
        let virtual_path = virtual_path.as_ref();
        let mut parts = virtual_path.split('/').collect::<VecDeque<_>>();

        let mut directory = &self.root;
        while parts.len() > 1 {
            let name = parts.pop_front().expect("failed to pop directory path");
            match directory
                .subdirectories
                .iter()
                .find(|sub| sub.name.eq_ignore_ascii_case(name))
            {
                Some(sub) => directory = sub,
                None => return None,
            }
        }

        if parts.len() == 1 {
            let name = parts.pop_front().expect("failed to pop node path");
            if let Some(node) = directory
                .nodes
                .iter()
                .find(|n| n.name.eq_ignore_ascii_case(name))
            {
                return Some(node);
            }
        }

        None
    }
}

impl Deserialize for Header {
    type Error = HeaderDeserializeError;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let magic_value = src.read_string(3)?;
        if magic_value != SAH_MAGIC_VALUE {
            return Err(HeaderDeserializeError::InvalidMagicValue {
                expected: SAH_MAGIC_VALUE.to_owned(),
                found: magic_value,
            });
        }

        let _header_version = src.read_u32::<byteorder::LittleEndian>()?;
        let _total_files = src.read_u32::<byteorder::LittleEndian>()?;
        src.skip(40)?;

        let root = VirtualDirectory::deserialize(src)?;
        Ok(Header { root })
    }
}

impl Deserialize for VirtualDirectory {
    type Error = std::io::Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let name = src.read_length_prefixed_string()?;
        let node_qty = src.read_u32::<byteorder::LittleEndian>()? as usize;
        let nodes = (0..node_qty)
            .map(|_| Inode::deserialize(src))
            .collect::<Result<Vec<Inode>, _>>()?;

        let subdirectory_qty = src.read_u32::<byteorder::LittleEndian>()? as usize;
        let subdirectories = (0..subdirectory_qty)
            .map(|_| VirtualDirectory::deserialize(src))
            .collect::<Result<Vec<VirtualDirectory>, _>>()?;

        Ok(VirtualDirectory {
            name,
            subdirectories,
            nodes,
        })
    }
}

impl Deserialize for Inode {
    type Error = std::io::Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let name = src.read_length_prefixed_string()?;
        let offset = src.read_u64::<byteorder::LittleEndian>()? as usize;
        let length = src.read_u32::<byteorder::LittleEndian>()? as usize;
        let checksum = src.read_u32::<byteorder::LittleEndian>()?;

        Ok(Inode {
            name,
            offset,
            length,
            checksum,
        })
    }
}
