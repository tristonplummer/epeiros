use crate::io::{Deserialize, Serialize};
use byteorder::{ReadBytesExt, WriteBytesExt};
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPublicKey};
use std::io::{Read, Write};

/// The expected capacity of the exponent. Regardless of the actual size of the exponent, the client
/// expects to be able to read 64 bytes.
const EXPONENT_CAPACITY: usize = 64;

/// The handshake request sent from the login server to the client upon first connecting.
#[derive(Debug, Clone)]
pub struct LoginHandshakeRequest {
    exponent: Vec<u8>,
    modulus: Vec<u8>,
}

impl LoginHandshakeRequest {
    /// Initialises a new handshake request, which contains the RSA public key
    /// that the client should respond with.
    ///
    /// # Arguments
    /// * `public_key`  - The public key.
    pub fn new(public_key: &RsaPublicKey) -> Self {
        let exponent = public_key.e().to_bytes_le();
        let modulus = public_key.n().to_bytes_le();

        Self { exponent, modulus }
    }

    /// Encrypts an input payload with the public key contained
    /// within this handshake request.
    ///
    /// # Arguments
    /// * `input`   - The payload to encrypt.
    pub fn encrypt(&self, input: &[u8]) -> Vec<u8> {
        let input = BigUint::from_bytes_le(input);
        let e = BigUint::from_bytes_le(&self.exponent);
        let n = BigUint::from_bytes_le(&self.modulus);

        input.modpow(&e, &n).to_bytes_le()
    }
}

impl Deserialize for LoginHandshakeRequest {
    type Error = std::io::Error;

    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let _enc_mode = src.read_u8()?;
        let expected_exponent_length = src.read_u8()? as usize;
        let modulus_length = src.read_u8()? as usize;

        let mut exponent = vec![0; EXPONENT_CAPACITY];
        let mut modulus = vec![0; modulus_length];

        let actual_exponent_length = src.read(&mut exponent)?;
        src.read_exact(&mut modulus)?;
        if actual_exponent_length != expected_exponent_length {
            tracing::error!(
                actual_exponent_length,
                expected_exponent_length,
                "mismatched exponent length"
            );
        }

        Ok(Self { exponent, modulus })
    }
}

impl Serialize for LoginHandshakeRequest {
    type Error = std::io::Error;

    fn serialize<T>(&self, dst: &mut T) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_u8(0)?; // byEncMode - always set to zero.
        dst.write_u8(self.exponent.len() as u8)?;
        dst.write_u8(self.modulus.len() as u8)?;

        // Shaiya is whack and expects to be able to read the exponent padded to 64 bytes, and
        // the modulus padded to 128 bytes. It will crash if this data is not available and valid.
        // Thankfully due to using a 1024-bit key, we don't need to care about padding the modulus.
        let mut padded_exponent = [0; EXPONENT_CAPACITY];
        padded_exponent[..self.exponent.len()].copy_from_slice(&self.exponent);
        dst.write_all(&padded_exponent)?;
        dst.write_all(&self.modulus)?;
        Ok(())
    }
}
