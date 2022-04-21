use std::fs::File;
use std::os::unix::fs::FileExt;

use crate::pk2::constants::{BLOCK_SIZE, BLOWFISH, ENTRY_SIZE};
use crate::pk2::entry::Entry;
use crate::pk2::errors::Error;
use crate::pk2::errors::Error::{InvalidBlock, IO};

/// Converts a byte slice in little endian to an u32 number.
pub fn as_u32_le(array: &[u8]) -> u32 {
    ((array[0] as u32) << 0) +
    ((array[1] as u32) << 8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}

/// Converts a byte slice in little endian to an u64 number.
pub fn as_u64_le(array: &[u8]) -> u64 {
    ((array[0] as u64) << 0) +
    ((array[1] as u64) << 8) +
    ((array[2] as u64) << 16) +
    ((array[3] as u64) << 24) +
    ((array[4] as u64) << 32) +
    ((array[5] as u64) << 40) +
    ((array[6] as u64) << 48) +
    ((array[7] as u64) << 56)
}

/// Reads a block (see [BLOCK_SIZE]) in a PK2 archive and returns the entries.
pub fn read_block(file: &File, offset: u64) -> Result<Vec<Entry>, Error> {
    let mut entry_buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    match file.read_at(&mut entry_buf, offset) {
        Err(err) => return Err(IO(err)),
        Ok(read_bytes) => if read_bytes % ENTRY_SIZE != 0 {
            return Err(InvalidBlock("wrong block size"))
        }
    }

    BLOWFISH.decrypt(&mut entry_buf);

    let mut entries: Vec<Entry> = entry_buf.chunks_exact(ENTRY_SIZE)
        .map(|buf| Entry::from(buf))
        .collect();

    if entries[19].next_chain > 0 {
        entries.extend(read_block(file, entries[19].next_chain)?.iter());
    }

    entries = entries.iter()
        .filter(|e| !e.is_empty())
        .map(|e| *e)
        .collect();

    Ok(entries)
}
