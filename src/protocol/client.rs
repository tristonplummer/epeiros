mod handshake;
mod login;

pub use handshake::*;
pub use login::*;

packets!(LoginClientPacket {
    LoginHandshakeResponse  => 0xA101,
    LoginRequest            => 0xA102,
});
