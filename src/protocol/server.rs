mod handshake;

pub use handshake::*;

packets!(LoginServerPacket {
    LoginHandshakeRequest   => 0xA101
});
