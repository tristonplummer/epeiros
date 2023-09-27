macro_rules! packets {
    (
        $ident:ident {
            $($packet:ident => $opcode:literal),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub enum $ident {
            $(
                $packet($packet),
            )*
        }

        impl $ident {
            pub fn opcode(&self) -> u16 {
                match self {
                    $(
                        $ident::$packet(_)  => $opcode,
                    )*
                }
            }
        }

        impl $crate::io::Deserialize for $ident {
            type Error = std::io::Error;

            fn deserialize<T: std::io::Read + byteorder::ReadBytesExt>(src: &mut T) -> Result<Self, std::io::Error>
            where
                Self: Sized
            {
                let opcode = src.read_u16::<byteorder::LittleEndian>()?;
                match opcode {
                    $(
                        opcode if opcode == $opcode => Ok($ident::$packet($packet::deserialize(src)?)),
                    )*
                    _ => Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("opcode does not exist: {:#06X}", opcode))),
                }
            }
        }

        impl $crate::io::Serialize for $ident {
            type Error = std::io::Error;

            fn serialize<T: std::io::Write + byteorder::WriteBytesExt>(&self, dst: &mut T) -> Result<(), Self::Error>
            {
                dst.write_u16::<byteorder::LittleEndian>(self.opcode())?;
                match self {
                    $(
                        $ident::$packet(packet) => {
                            packet.serialize(dst)?;
                        }
                    )*
                }
                Ok(())
            }
        }

        $(
            impl From<$packet> for $ident {
                fn from(packet: $packet) -> Self {
                    $ident::$packet(packet)
                }
            }
        )*
    };
}

pub mod client;
pub mod server;
