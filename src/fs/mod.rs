use crate::fs::header::{Header, HeaderDeserializeError};
use crate::io::{Deserialize, GameVersion};
use memmap2::Mmap;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

pub mod header;
pub mod types;

trait ReadableStorage {
    fn read<T>(&self, virtual_path: T) -> Option<Vec<u8>>
    where
        T: AsRef<str>;

    fn read_type<T>(&self, virtual_path: impl AsRef<str>) -> Result<T, std::io::Error>
    where
        T: Deserialize<Error = std::io::Error>,
    {
        self.read_versioned_type(virtual_path, GameVersion::Ep4)
    }

    fn read_versioned_type<T>(
        &self,
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

pub struct ImmutableFilestore {
    header: Header,
    data_file: Mmap,
}

impl ImmutableFilestore {
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
    fn read<T>(&self, virtual_path: T) -> Option<Vec<u8>>
    where
        T: AsRef<str>,
    {
        if let Some(node) = self.header.get_inode(virtual_path) {
            let end_offset = node.offset + node.length;
            let slice = &self.data_file[node.offset..end_offset];
            return Some(Vec::from(slice));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::types::SkillData;
    use crate::fs::{ImmutableFilestore, ReadableStorage};
    use crate::io::{Deserialize, GameVersion, Serialize};
    use std::io::Cursor;

    #[test]
    fn test() {
        let fs = ImmutableFilestore::open("res/data.sah", "res/data.saf")
            .expect("failed to open filestore");
        assert!(fs.read_type::<String>("filter.txt").is_ok())
    }

    #[test]
    fn sdata_write() {
        let fs = ImmutableFilestore::open("res/data.sah", "res/data.saf")
            .expect("failed to open filestore");
        let character_skills = fs
            .read_versioned_type::<SkillData>("character/skill.sdata", GameVersion::Ep6)
            .unwrap();

        let mut out = Vec::new();
        character_skills
            .versioned_serialize(&mut out, GameVersion::Ep6)
            .unwrap();

        std::fs::write("res/Skill.SData", &out).unwrap();
    }

    #[test]
    fn sdata_read() {
        let fs = ImmutableFilestore::open("res/data.sah", "res/data.saf")
            .expect("failed to open filestore");

        let mut indata = std::fs::read("res/Skill.SData").unwrap();
        let mut src = Cursor::new(indata.as_slice());
        let sdata = SkillData::versioned_deserialize(&mut src, GameVersion::Ep6).unwrap();

        std::fs::write(
            "res/skillsout.json",
            serde_json::to_vec_pretty(&sdata).unwrap(),
        )
        .unwrap();
    }
}
