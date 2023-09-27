mod handshake;

pub use handshake::*;

packets!(LoginClientPacket {
    LoginHandshakeResponse    => 0xA101
});
