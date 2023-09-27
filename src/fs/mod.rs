use crate::fs::header::{Header, HeaderDeserializeError, Inode};
use crate::io::{Deserialize, GameVersion, Serialize};
use crc32fast::Hasher;
use memmap2::Mmap;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub mod header;
pub mod types;

pub trait ReadableStorage {
    /// Get the path to every node contained within the storage.
    fn all_node_paths(&self) -> Vec<String>;

    /// Reads a file at a given path in the virtual filesystem. This will return `None` if a node
    /// is not found at the given path.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the file, relative to the root directory.
    fn read<T>(&mut self, virtual_path: T) -> Option<Vec<u8>>
    where
        T: AsRef<str>;

    /// Deserializes a file at a given path in the filesystem. This will attempt to deserialize
    /// with every game version, until it either fails or finds a match. If you know the relevant
    /// [GameVersion] before hand, please use [Self::read_versioned_type] and specify it
    /// explicitly.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the file, relative to the root directory.
    fn read_type<T>(
        &mut self,
        virtual_path: impl AsRef<str>,
    ) -> Result<(T, GameVersion), std::io::Error>
    where
        T: Deserialize<Error = std::io::Error>,
    {
        for version in GameVersion::all() {
            if let Ok(data) = self.read_versioned_type(virtual_path.as_ref(), *version) {
                return Ok((data, *version));
            }
        }
        Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "did not match any game version",
        ))
    }

    /// Deserializes a file at a given path in the filesystem, using a specified [GameVersion]. This will
    /// not attempt to parse the file with any other versions, and should only be used in a scenario where you
    /// know the encoded version beforehand.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the file, relative to the root directory.
    /// * `version`         - The game version.
    fn read_versioned_type<T>(
        &mut self,
        virtual_path: impl AsRef<str>,
        version: GameVersion,
    ) -> Result<T, std::io::Error>
    where
        T: Deserialize<Error = std::io::Error>,
    {
        match self.read(virtual_path) {
            Some(mut data) => {
                let mut src = Cursor::new(&mut data);
                T::versioned_deserialize(&mut src, version)
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "inode does not exist",
            )),
        }
    }
}

pub trait WritableStorage {
    /// Writes some data to a node at a given path. If the node does not already exist, it will
    /// be created.
    ///
    /// # Arguments
    /// * `virtual_path`        - The path to file node.
    /// * `data`                - The data to write.
    /// * `serialize_header`    - If the header should be serialized. This should be set to false when writing multiple files.
    fn write<T>(
        &mut self,
        virtual_path: T,
        data: &[u8],
        serialize_header: bool,
    ) -> Result<(), std::io::Error>
    where
        T: AsRef<str>;

    /// Writes a serializable type to a node at a given path. If the node does not already exist, it
    /// will be created. The type will be serialized with the latest [GameVersion] using [GameVersion::last].
    /// If you already know what version the data should support, please use [Self::write_versioned_type]
    /// with an explicit version.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the file node.
    /// * `typ`             - The serializable type.
    fn write_type<T>(
        &mut self,
        virtual_path: impl AsRef<str>,
        typ: &T,
    ) -> Result<(), std::io::Error>
    where
        T: Serialize<Error = std::io::Error>,
    {
        self.write_versioned_type(virtual_path, typ, *GameVersion::last())
    }

    /// Writes a serializable type to a given at a given path. If the node does not already exist, it
    /// will be created.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the file node.
    /// * `typ`             - The serializable type.
    /// * `version`         - The game version used for serialization.
    fn write_versioned_type<T>(
        &mut self,
        virtual_path: impl AsRef<str>,
        typ: &T,
        version: GameVersion,
    ) -> Result<(), std::io::Error>
    where
        T: Serialize<Error = std::io::Error>,
    {
        let mut dst = Vec::with_capacity(10_000);
        typ.versioned_serialize(&mut dst, version)?;

        self.write(virtual_path, &dst, true)
    }
}

/// An efficient, read-only view over a filestore. This will not allow any files to be modified, and
/// is backed by a memory-mapped view of the data file.
pub struct ImmutableFilestore {
    header: Header,
    data_file: Mmap,
}

/// A filestore which supports both reading and writing of files. This uses traditional disk I/O. If
/// only reading is required, consider using [ImmutableFilestore] as it will read data much more
/// quickly.
pub struct MutableFilestore {
    header_file: File,
    header: Header,
    data_file: File,
}

impl ImmutableFilestore {
    /// Opens a filestore from a known header and data file path.
    ///
    /// # Errors
    /// Returns an error if either of the files don't exist, or if the header
    /// cannot be parsed.
    ///
    /// # Arguments
    /// * `header_path` - The path to the header file.
    /// * `data_path`   - The path to the data file.
    pub fn open<P>(header_path: P, data_path: P) -> Result<Self, HeaderDeserializeError>
    where
        P: AsRef<Path>,
    {
        let header_path = header_path.as_ref();
        let data_path = data_path.as_ref();

        let header = Header::open(header_path)?;
        let data_file = unsafe { Mmap::map(&File::open(data_path)?) }?;
        Ok(Self { header, data_file })
    }
}

impl ReadableStorage for ImmutableFilestore {
    /// Get the path to every node contained within the storage.
    fn all_node_paths(&self) -> Vec<String> {
        self.header.get_all_node_paths()
    }

    /// Reads a file at a given path in the virtual filesystem. This will return `None` if a node
    /// is not found at the given path.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the file, relative to the root directory.
    fn read<T>(&mut self, virtual_path: T) -> Option<Vec<u8>>
    where
        T: AsRef<str>,
    {
        if let Some(node) = self.header.get_inode(&virtual_path) {
            let end_offset = node.offset + node.length;
            let slice = &self.data_file[node.offset..end_offset];
            return Some(Vec::from(slice));
        }

        None
    }
}

impl MutableFilestore {
    /// Opens an existing filestore from a header and data file path. If you don't need to
    /// write to this filestore, consider using [ImmutableFilestore::open] instead.
    ///
    /// # Errors
    /// Returns an error if either of the files don't exist, or if the header
    /// cannot be parsed.
    ///
    /// # Arguments
    /// * `header_path` - The path to the header file.
    /// * `data_path`   - The path to the data file.
    pub fn open<P>(header_path: P, data_path: P) -> Result<Self, HeaderDeserializeError>
    where
        P: AsRef<Path>,
    {
        let header_path = header_path.as_ref();
        let data_path = data_path.as_ref();

        let header = Header::open(header_path)?;
        let header_file = OpenOptions::new().write(true).open(header_path)?;
        let data_file = OpenOptions::new().read(true).write(true).open(data_path)?;

        Ok(Self {
            header_file,
            header,
            data_file,
        })
    }

    /// Creates an empty filestore at a given path. If files already exist at the specified paths,
    /// they will be overwritten. This is useful for creating patches from flat files. If you need to
    /// update an existing filestore without losing the data, use [Self::open].
    ///
    /// # Arguments
    /// * `header_path` - The path where the header file should be created.
    /// * `data_path`   - The path where the data file should be created.
    pub fn create<P>(header_path: P, data_path: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let header_path = header_path.as_ref();
        let data_path = data_path.as_ref();

        let header_file = File::create(header_path)?;
        let data_file = File::create(data_path)?;

        Ok(Self {
            header_file,
            header: Header::default(),
            data_file,
        })
    }

    /// Serializes the header view to the backing file.
    fn serialize_header(&mut self) -> Result<(), std::io::Error> {
        let mut dst = Vec::with_capacity(4_000_000);
        self.header.serialize(&mut dst)?;

        self.header_file.set_len(0)?;
        self.header_file.write_all(&dst)?;
        Ok(())
    }

    /// Patches this filestore by taking every file from `other`, and placing it at the same path
    /// in this filestore.
    ///
    /// # Arguments
    /// * `other`   - The storage to read from.
    pub fn patch(&mut self, other: &mut impl ReadableStorage) -> Result<(), std::io::Error> {
        let other_nodes = other.all_node_paths();
        for node in &other_nodes {
            let data = other
                .read(node)
                .expect("failed to read known node in other storage");
            self.write(node, &data, false)?;
        }

        self.serialize_header()?;
        Ok(())
    }
}

impl ReadableStorage for MutableFilestore {
    /// Get the path to every node contained within the storage.
    fn all_node_paths(&self) -> Vec<String> {
        self.header.get_all_node_paths()
    }

    /// Reads a file at a given path in the virtual filesystem. This will return `None` if a node
    /// is not found at the given path.
    ///
    /// # Arguments
    /// * `virtual_path`    - The path to the file, relative to the root directory.
    fn read<T>(&mut self, virtual_path: T) -> Option<Vec<u8>>
    where
        T: AsRef<str>,
    {
        if let Some(node) = self.header.get_inode(&virtual_path) {
            if self
                .data_file
                .seek(SeekFrom::Start(node.offset as u64))
                .is_err()
            {
                return None;
            }

            let mut data = vec![0; node.length];
            return match self.data_file.read(data.as_mut_slice()) {
                Ok(_) => Some(data),
                Err(_) => None,
            };
        }

        None
    }
}

impl WritableStorage for MutableFilestore {
    /// Writes some data to a node at a given path. If the node does not already exist, it will
    /// be created.
    ///
    /// # Arguments
    /// * `virtual_path`        - The path to file node.
    /// * `data`                - The data to write.
    /// * `serialize_header`    - If the header should be serialized. This should be set to false when writing multiple files.
    fn write<T>(
        &mut self,
        virtual_path: T,
        data: &[u8],
        serialize_header: bool,
    ) -> Result<(), std::io::Error>
    where
        T: AsRef<str>,
    {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let checksum = hasher.finalize();

        if let Some(inode) = self.header.get_inode_mut(&virtual_path) {
            let existing_space = inode.length;
            let can_fit_into_existing_space = data.len() <= existing_space;
            inode.length = data.len();
            inode.checksum = checksum;

            if can_fit_into_existing_space {
                self.data_file.seek(SeekFrom::Start(inode.offset as u64))?;

                let mut file_buf = vec![0; existing_space];
                file_buf[..data.len()].copy_from_slice(data);
                self.data_file.write_all(&file_buf)?;
            } else {
                let offset = self.data_file.seek(SeekFrom::End(0))?;
                inode.offset = offset as usize;

                self.data_file.set_len(offset + (inode.length as u64))?;
                self.data_file.write_all(data)?;
            }
            if serialize_header {
                self.serialize_header()?;
            }

            return Ok(());
        }

        let virtual_path = virtual_path.as_ref();
        let name = virtual_path.split('/').last().unwrap();
        let offset = self.data_file.seek(SeekFrom::End(0))?;

        let inode = Inode {
            name: name.to_owned(),
            offset: offset as usize,
            length: data.len(),
            checksum,
        };

        self.data_file.set_len(offset + (inode.length as u64))?;
        self.data_file.write_all(data)?;

        self.header.emplace_node(virtual_path, inode)?;

        if serialize_header {
            self.serialize_header()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::types::{ItemData, SkillData};
    use crate::fs::{ImmutableFilestore, MutableFilestore, ReadableStorage};
    use crate::io::{Deserialize, GameVersion, Serialize};
    use std::io::Cursor;

    #[test]
    fn test() {
        let mut fs = ImmutableFilestore::open("res/data.sah", "res/data.saf")
            .expect("failed to open filestore");
        assert!(fs.read_type::<String>("filter.txt").is_ok())
    }

    #[test]
    fn sdata_write() {
        let mut fs = ImmutableFilestore::open("res/data.sah", "res/data.saf")
            .expect("failed to open filestore");
        let (items, _version) = fs.read_type::<ItemData>("item/item.sdata").unwrap();

        std::fs::write("res/items.json", serde_json::to_vec_pretty(&items).unwrap()).unwrap();
        let mut out = Vec::new();
        items
            .versioned_serialize(&mut out, GameVersion::Ep6)
            .unwrap();

        std::fs::write("res/Item.SData", &out).unwrap();

        let skills = fs
            .read_versioned_type::<SkillData>("character/skill.sdata", GameVersion::Ep6)
            .unwrap();

        std::fs::write(
            "res/skills.json",
            serde_json::to_vec_pretty(&skills).unwrap(),
        )
        .unwrap();
        let mut out = Vec::new();
        skills
            .versioned_serialize(&mut out, GameVersion::Ep6)
            .unwrap();

        std::fs::write("res/Skill.SData", &out).unwrap();
    }

    #[test]
    fn sdata_read() {
        let indata = std::fs::read("res/Skill.SData").unwrap();
        let mut src = Cursor::new(indata.as_slice());
        let sdata = SkillData::versioned_deserialize(&mut src, GameVersion::Ep6).unwrap();

        std::fs::write(
            "res/skillsout.json",
            serde_json::to_vec_pretty(&sdata).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn item_test() {
        let item = std::fs::read("res/KreonItem.SData").unwrap();
        let mut src = Cursor::new(item.as_slice());

        let items = ItemData::versioned_deserialize(&mut src, GameVersion::Ep6v2).unwrap();
        std::fs::write(
            "res/kreonitems.json",
            serde_json::to_vec_pretty(&items).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn patch_test() {
        let mut other = ImmutableFilestore::open("res/data.sah", "res/data.saf").unwrap();
        let mut fs =
            MutableFilestore::create("res/new_with_crc.sah", "res/new_with_crc.saf").unwrap();
        fs.patch(&mut other).unwrap();
    }
}
