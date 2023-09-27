mod handshake;
mod login;
mod serverlist;

pub use handshake::*;
pub use login::*;
pub use serverlist::*;

packets!(LoginServerPacket {
    LoginHandshakeRequest   => 0xA101,
    LoginResponse           => 0xA102,
    ServerList              => 0xA201,
});
