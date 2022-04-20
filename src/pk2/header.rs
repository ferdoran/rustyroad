use crate::blowfish::Blowfish;
use crate::pk2::constants::{CHECKSUM, HEADER_SIZE, KEY_BYTES, SALT, SIGNATURE, VERSION};
use crate::pk2::errors::Error;
use crate::pk2::errors::Error::InvalidHeader;
use crate::pk2::util::as_u32_le;

pub struct Header {
    signature: [u8; 30],
    version: u32,
    encrypted: bool,
    checksum: [u8; 16],
    reserved: [u8; 205]
}


impl From<[u8; HEADER_SIZE]> for Header {
    fn from(buf: [u8; HEADER_SIZE]) -> Self {
        let mut header = Header{
            signature: [0; 30],
            version: as_u32_le(&buf[30..34]),
            encrypted: buf[34] == 1,
            checksum: [0; 16],
            reserved: [0; 205]
        };

        header.signature.copy_from_slice(&buf[0..30]);
        header.checksum.copy_from_slice(&buf[35..51]);
        header.reserved.copy_from_slice(&buf[51..]);

        return header;
    }
}

impl From<&[u8]> for Header {
    fn from(header_buf: &[u8]) -> Self {
        if header_buf.len() < HEADER_SIZE {
            panic!("header length too short");
        }

        let mut buf = [0; HEADER_SIZE];
        buf.copy_from_slice(header_buf);
        return Header::from(buf);
    }
}

impl Header {
    fn verify_checksum(&self) -> Result<(), Error> {
        if !self.encrypted {
            return Ok(());
        }

        let bf = Blowfish::new(KEY_BYTES, SALT).unwrap();
        let mut encrypted_checksum = CHECKSUM.clone();
        bf.encrypt(&mut encrypted_checksum);

        for i in 0..3 {
            if encrypted_checksum[i] != self.checksum[i] {
                return Err(InvalidHeader("Checksum is invalid"));
            }
        }

        return Ok(());
    }

    fn verify_signature(&self) -> Result<(), Error> {
        if &self.signature != SIGNATURE {
            Err(InvalidHeader("Invalid signature"))
        } else if self.version != VERSION {
            Err(InvalidHeader("Invalid version"))
        } else {
            Ok(())
        }
    }

    pub fn verify(&self) -> Result<(), Error> {
        self.verify_signature()?;
        self.verify_checksum()
    }
}