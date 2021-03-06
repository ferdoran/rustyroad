use lazy_static::lazy_static;
use crate::blowfish::Blowfish;

pub const ENTRY_SIZE: usize = 128;
pub const BLOCK_SIZE: usize = 20 * ENTRY_SIZE;
pub const HEADER_SIZE: usize = 256;
pub const KEY: &str = "169841";
pub const KEY_BYTES: &[u8] = KEY.as_bytes();
pub const SALT: &[u8; 10] = &[0x03, 0xF8, 0xE4, 0x44, 0x88, 0x99, 0x3F, 0x64, 0xFE, 0x35];
pub const CHECKSUM: &[u8; 16] = b"Joymax Pak File\0";
pub const SIGNATURE: &[u8; 30] = b"JoyMax File Manager!\x0a\x00\x00\x00\x00\x00\x00\x00\x00\x00";
pub const VERSION: u32 = 0x0100_0002;

lazy_static! {
    pub static ref BLOWFISH: Blowfish = Blowfish::new(KEY_BYTES, SALT).expect("blowfish failed to init");
}
