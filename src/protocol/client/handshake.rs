use crate::io::{Deserialize, Serialize};
use aes::Aes128;
use byteorder::{ReadBytesExt, WriteBytesExt};
use cipher::KeyIvInit;
use ctr::Ctr128LE;
use hmac::{Hmac, Mac};
use rsa::traits::{PrivateKeyParts, PublicKeyParts};
use rsa::{BigUint, RsaPrivateKey};
use sha2::Sha256;
use std::io::{Read, Write};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;
type Aes128Ctr = Ctr128LE<Aes128>;

/// The handshake response sent from the client.
#[derive(Debug, Clone)]
pub struct LoginHandshakeResponse {
    pub payload: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum HandshakeResponseError {
    #[error("payload length did not match modulus (expected {expected:?}, got {actual:?})")]
    InvalidPayloadLength { expected: usize, actual: usize },
}

impl LoginHandshakeResponse {
    /// Decrypts the handshake response payload with a private key. This will throw an error
    /// if the length of the payload does not match that of the modulus.
    ///
    /// # Arguments
    /// * `private_key` - The private key.
    fn decrypt(&self, private_key: &RsaPrivateKey) -> Result<Vec<u8>, HandshakeResponseError> {
        let n = private_key.n();
        let d = private_key.d();

        let payload_len = self.payload.len();
        let mod_len = n.to_bytes_le().len();
        if payload_len != mod_len {
            return Err(HandshakeResponseError::InvalidPayloadLength {
                expected: mod_len,
                actual: payload_len,
            });
        }

        let encrypted = BigUint::from_bytes_le(&self.payload);
        let decrypted = encrypted.modpow(d, n);
        Ok(decrypted.to_bytes_le())
    }

    /// Decrypts the handshake response payload and performs a HmacSHA256 function to
    /// generate an AES key pair.
    ///
    /// # Arguments
    /// * `private_key` - The private key.
    pub fn to_aes_keypair(
        &self,
        private_key: &RsaPrivateKey,
    ) -> Result<(Aes128Ctr, Aes128Ctr), Box<dyn std::error::Error>> {
        let n = private_key.n().to_bytes_le();
        let payload = self.decrypt(private_key)?;
        let mut mac = HmacSha256::new_from_slice(&payload)?;
        mac.update(&n);

        let key_base = mac.finalize().into_bytes();
        let (key, iv) = key_base.split_at(16);

        let send_cipher = Aes128Ctr::new(key.into(), iv.into());
        let recv_cipher = Aes128Ctr::new(key.into(), iv.into());
        Ok((send_cipher, recv_cipher))
    }
}

impl Serialize for LoginHandshakeResponse {
    type Error = std::io::Error;

    /// Writes the handshake response to the server.
    ///
    /// # Arguments
    /// * `dst` - The destination buffer.
    fn serialize<T>(&self, dst: &mut T) -> Result<(), Self::Error>
    where
        T: Write + WriteBytesExt,
    {
        dst.write_u8(self.payload.len() as u8)?;
        dst.write_all(&self.payload)?;
        Ok(())
    }
}

impl Deserialize for LoginHandshakeResponse {
    type Error = std::io::Error;

    /// Reads a login handshake response from a client.
    ///
    /// # Arguments
    /// * `src` - The source buffer.
    fn deserialize<T>(src: &mut T) -> Result<Self, Self::Error>
    where
        T: Read + ReadBytesExt,
        Self: Sized,
    {
        let payload_length = src.read_u8()? as usize;
        let mut payload = vec![0; payload_length];
        src.read_exact(&mut payload)?;

        Ok(Self { payload })
    }
}
