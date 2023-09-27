use byteorder::WriteBytesExt;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub enum GameVersion {
    Ep4,
    Ep5,
    Ep6,
    Ep6v2,
}

pub trait Serialize {
    type Error;

    fn serialize<T>(&self, dst: &mut T) -> Result<(), Self::Error>
    where
        T: std::io::Write + WriteBytesExt,
    {
        self.versioned_serialize(dst, GameVersion::Ep4)
    }

    fn versioned_serialize<T>(&self, dst: &mut T, version: GameVersion) -> Result<(), Self::Error>
    where
        T: std::io::Write + WriteBytesExt;
}

pub trait Deserialize {
    type Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: std::io::Read + byteorder::ReadBytesExt,
        Self: Sized,
    {
        Self::versioned_deserialize(src, GameVersion::Ep4)
    }

    fn versioned_deserialize<T>(src: &mut T, version: GameVersion) -> Result<Self, Self::Error>
    where
        T: std::io::Read + byteorder::ReadBytesExt,
        Self: Sized;
}

pub trait ShaiyaReadExt {
    type Error;

    fn consume_all(&mut self) -> Vec<u8>;

    fn read_string(&mut self, length: usize) -> Result<String, Self::Error>;

    fn read_length_prefixed_string(&mut self) -> Result<String, Self::Error>;

    fn skip(&mut self, length: usize) -> Result<(), Self::Error>;
}

pub trait ShaiyaWriteExt {
    type Error;

    fn write_string<T>(&mut self, text: T, length: usize) -> Result<(), Self::Error>
    where
        T: AsRef<str>;

    fn write_length_prefixed_string<T>(&mut self, text: T) -> Result<(), Self::Error>
    where
        T: AsRef<str>;

    fn write_bool(&mut self, value: bool) -> Result<(), Self::Error>;
}

impl<R> ShaiyaReadExt for R
where
    R: std::io::Read + byteorder::ReadBytesExt,
{
    type Error = std::io::Error;

    fn consume_all(&mut self) -> Vec<u8> {
        let mut dst = Vec::new();
        while let Ok(b) = self.read_u8() {
            dst.push(b);
        }

        dst
    }

    fn read_string(&mut self, length: usize) -> Result<String, Self::Error> {
        let mut dst = vec![0; length];
        self.read_exact(&mut dst)?;

        let mut text = String::with_capacity(length);
        for ch in dst.iter() {
            if *ch == 0 {
                break;
            }

            text.push(char::from(*ch));
        }

        Ok(text)
    }

    fn read_length_prefixed_string(&mut self) -> Result<String, Self::Error> {
        let length = self.read_u32::<byteorder::LittleEndian>()? as usize;
        self.read_string(length)
    }

    fn skip(&mut self, length: usize) -> Result<(), Self::Error> {
        let mut dst = vec![0; length];
        self.read_exact(&mut dst)?;
        Ok(())
    }
}

impl<W> ShaiyaWriteExt for W
where
    W: std::io::Write + byteorder::WriteBytesExt,
{
    type Error = std::io::Error;

    fn write_string<T>(&mut self, text: T, length: usize) -> Result<(), Self::Error>
    where
        T: AsRef<str>,
    {
        let mut dst = vec![0; length];
        let bytes = text.as_ref().as_bytes();
        dst[..bytes.len()].copy_from_slice(bytes);

        if let Some(last) = dst.last_mut() {
            *last = 0;
        }

        self.write_all(&dst)?;
        Ok(())
    }

    fn write_length_prefixed_string<T>(&mut self, text: T) -> Result<(), Self::Error>
    where
        T: AsRef<str>,
    {
        let text = text.as_ref();
        let length_with_null_terminator = text.bytes().len() + 1;

        self.write_u32::<byteorder::LittleEndian>(length_with_null_terminator as u32)?;
        self.write_string(text, length_with_null_terminator)?;
        Ok(())
    }

    fn write_bool(&mut self, value: bool) -> Result<(), Self::Error> {
        self.write_u8(if value { 1 } else { 0 })
    }
}
