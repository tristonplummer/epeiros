mod handshake;
mod login;

pub use handshake::*;
pub use login::*;

packets!(LoginServerPacket {
    LoginHandshakeRequest   => 0xA101,
    LoginResponse           => 0xA102,
});
