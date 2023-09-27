pub trait Serialize {
    type Error;

    fn serialize<T>(&self, dst: &mut T) -> Result<(), Self::Error>
    where
        T: std::io::Write + byteorder::WriteBytesExt;
}

pub trait Deserialize {
    type Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
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

impl<T> ShaiyaReadExt for T
where
    T: std::io::Read + byteorder::ReadBytesExt,
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
