use crate::io::{Deserialize, GameVersion, Serialize, ShaiyaReadExt, ShaiyaWriteExt};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::collections::VecDeque;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use thiserror::Error;

/// The magic value which always appears at the start of the header file.
const SAH_MAGIC_VALUE: &str = "SAH";

/// The header file format version. This is always set to zero.
const HEADER_FORMAT_VERSION: u32 = 0;

/// A listing of every [VirtualDirectory] and every [Inode] contained within the virtual
/// filesystem. This is serialized to a `.SAH` file and used by the game client to look up files
/// by their path, relative to the root directory.
pub struct Header {
    root: VirtualDirectory,
}

/// A directory within the virtual filesystem, which may contain file nodes and other directories.
pub struct VirtualDirectory {
    pub name: String,
    pub subdirectories: Vec<VirtualDirectory>,
    pub nodes: Vec<Inode>,
}

/// Represents a file node in the filesystem's corresponding data file.
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
    /// Opens and parses an existing header file. This will fail if the header file does not already
    /// exist, or if an error occurs during parsing. For creating an empty header, please use
    /// `Header::default()`.
    ///
    /// # Arguments
    /// * `path`    - The path to the header file.
    pub fn open<P>(path: P) -> Result<Self, HeaderDeserializeError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let buf = std::fs::read(path)?;

        let mut src = Cursor::new(buf.as_slice());
        Self::deserialize(&mut src)
    }

    /// Recursively gets the path to all nodes in the header. This is useful for getting
    /// a listing of everything contained within the filesystem.
    pub fn get_all_node_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for inode in &self.root.nodes {
            paths.push(inode.name.clone());
        }

        for subdir in &self.root.subdirectories {
            paths.extend(subdir.node_paths());
        }

        paths
    }

    /// Gets an inode contained within the filesystem. This will return `None` if either a directory
    /// or the file does not exist. The path is case-insensitive.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the node.
    pub fn get_inode<T>(&self, virtual_path: &T) -> Option<&Inode>
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

    /// Gets an inode contained within the filesystem. This will return `None` if either a directory
    /// or the file does not exist. The path is case-insensitive.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the node.
    pub fn get_inode_mut<T>(&mut self, virtual_path: &T) -> Option<&mut Inode>
    where
        T: AsRef<str>,
    {
        let virtual_path = virtual_path.as_ref();
        let mut parts = virtual_path.split('/').collect::<VecDeque<_>>();

        let mut directory = &mut self.root;
        while parts.len() > 1 {
            let name = parts.pop_front().expect("failed to pop directory path");
            if directory.subdirectory_exists(name) {
                directory = directory
                    .get_subdirectory(name)
                    .expect("failed to get known existing subdirectory");
            } else {
                return None;
            }
        }

        if parts.len() == 1 {
            let name = parts.pop_front().expect("failed to pop node path");
            if let Some(node) = directory
                .nodes
                .iter_mut()
                .find(|n| n.name.eq_ignore_ascii_case(name))
            {
                return Some(node);
            }
        }

        None
    }

    /// Places an [Inode] into an appropriate position in this filesystem. If the subdirectories
    /// required to reach this node do not yet exist, they will be allocated.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to place the node.
    /// * `node`            - The node data.
    pub fn emplace_node<T>(&mut self, virtual_path: T, node: Inode) -> Result<(), std::io::Error>
    where
        T: AsRef<str>,
    {
        let virtual_path = virtual_path.as_ref();
        let mut parts = virtual_path.split('/').collect::<VecDeque<_>>();

        let mut directory = &mut self.root;
        while parts.len() > 1 {
            let name = parts.pop_front().expect("failed to pop directory path");
            if directory.subdirectory_exists(name) {
                directory = directory
                    .get_subdirectory(name)
                    .expect("failed to get known existing subdirectory");
            } else {
                directory = directory.create_and_get_subdirectory(name);
            }
        }

        assert_eq!(parts.len(), 1);
        directory.nodes.push(node);
        Ok(())
    }
}

impl Default for Header {
    fn default() -> Self {
        let root = VirtualDirectory {
            name: String::default(),
            subdirectories: Vec::new(),
            nodes: Vec::new(),
        };

        Self { root }
    }
}

impl VirtualDirectory {
    /// Gets a subdirectory contained within this directory by name. This does not
    /// recurse through directories to find it.
    ///
    /// # Arguments
    /// * `name`    - The name of the subdirectory.
    fn get_subdirectory(&mut self, name: &str) -> Option<&mut VirtualDirectory> {
        self.subdirectories
            .iter_mut()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// Checks if a subdirectory with a given name exists within this directory. It does not
    /// recurse through directories.
    ///
    /// # Arguments
    /// * `name`    - The name of the subdirectory.
    fn subdirectory_exists(&self, name: &str) -> bool {
        self.subdirectories
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// Creates an empty subdirectory with a given name, and returns it.
    ///
    /// # Arguments
    /// * `name`    - The name of the subdirectory.
    fn create_and_get_subdirectory(&mut self, name: &str) -> &mut VirtualDirectory {
        self.create_subdirectory(name);
        self.get_subdirectory(name)
            .expect("failed to get previously created subdirectory")
    }

    /// Creates an empty subdirectory with a given name.
    ///
    /// # Arguments
    /// * `name`    - The name of the subdirectory.
    fn create_subdirectory(&mut self, name: &str) {
        assert!(!self.subdirectory_exists(name));
        let subdirectory = VirtualDirectory {
            name: name.to_string(),
            subdirectories: Vec::new(),
            nodes: Vec::new(),
        };
        self.subdirectories.push(subdirectory);
    }

    /// Recursively gets the path to every node in this directory, and all of it's subdirectories.
    fn node_paths(&self) -> Vec<String> {
        let mut paths = Vec::with_capacity(self.nodes.len());
        for inode in &self.nodes {
            paths.push(format!("{}/{}", &self.name, inode.name));
        }

        for subdir in &self.subdirectories {
            let child_nodes = subdir.node_paths();
            for inode in &child_nodes {
                paths.push(format!("{}/{}", &self.name, inode));
            }
        }

        paths
    }
}

impl Deserialize for Header {
    type Error = HeaderDeserializeError;

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
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

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
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

    fn versioned_deserialize<T>(src: &mut T, _version: GameVersion) -> Result<Self, Self::Error>
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

impl Serialize for Header {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        let inode_qty = self.get_all_node_paths().len();

        dst.write_string(SAH_MAGIC_VALUE, 3)?;
        dst.write_u32::<byteorder::LittleEndian>(HEADER_FORMAT_VERSION)?;
        dst.write_u32::<byteorder::LittleEndian>(inode_qty as u32)?;
        let padding = vec![0; 40];
        dst.write_all(&padding)?;
        self.root.versioned_serialize(dst, version)?;
        dst.write_u64::<byteorder::LittleEndian>(0)?;
        Ok(())
    }
}

impl Serialize for VirtualDirectory {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_length_prefixed_string(&self.name)?;

        dst.write_u32::<byteorder::LittleEndian>(self.nodes.len() as u32)?;
        for node in &self.nodes {
            node.versioned_serialize(dst, version)?;
        }

        dst.write_u32::<byteorder::LittleEndian>(self.subdirectories.len() as u32)?;
        for subdir in &self.subdirectories {
            subdir.versioned_serialize(dst, version)?;
        }
        Ok(())
    }
}

impl Serialize for Inode {
    type Error = std::io::Error;

    fn versioned_serialize<T>(&self, dst: &mut T, _version: GameVersion) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_length_prefixed_string(&self.name)?;
        dst.write_u64::<byteorder::LittleEndian>(self.offset as u64)?;
        dst.write_u32::<byteorder::LittleEndian>(self.length as u32)?;
        dst.write_u32::<byteorder::LittleEndian>(self.checksum)?;
        Ok(())
    }
}
