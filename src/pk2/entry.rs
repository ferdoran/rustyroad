use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

use crate::pk2::constants::ENTRY_SIZE;
use crate::pk2::util::{as_u32_le, as_u64_le};

/// Byte representation of an [Entry]'s type.
#[repr(u8)]
#[derive(PartialEq)]
pub enum EntryType {
    Empty = 0,
    Dir = 1,
    File = 2
}

impl Display for EntryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::Empty => f.write_str("Empty"),
            EntryType::Dir => f.write_str("Directory"),
            EntryType::File => f.write_str("File")
        }
    }
}

impl From<u8> for EntryType {
    fn from(val: u8) -> Self {
        return match val {
            0 => EntryType::Empty,
            1 => EntryType::Dir,
            2 => EntryType::File,
            _ => panic!("{} is not a correct entry type value", val),
        }
    }
}

/// Raw entry structure in the PK2 archive. Can represent a directory, file or empty entry (see [EntryType]).
#[derive(Copy, Clone)]
pub struct Entry {
    pub typ: u8,
    pub name: [u8; 89],
    create_time: u64,
    modify_time: u64,
    pub position: u64,
    pub size: u32,
    pub next_chain: u64,
    padding: [u8; 2]
}

impl From<&[u8]> for Entry {
    fn from(buf: &[u8]) -> Self {
        if buf.len() != ENTRY_SIZE {
            panic!("invalid buffer size: {}", buf.len());
        }

        let mut entry = Entry {
            typ: buf[0],
            name: [0; 89],
            create_time: as_u64_le(&buf[90..98]),
            modify_time: as_u64_le(&buf[98..106]),
            position: as_u64_le(&buf[106..114]),
            size: as_u32_le(&buf[114..118]),
            next_chain: as_u64_le(&buf[118..126]),
            padding: [0; 2]
        };

        entry.name.copy_from_slice(&buf[1..90]);
        entry.padding.copy_from_slice(&buf[126..128]);

        return entry;
    }
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        self.typ == 1
    }
    pub fn is_file(&self) -> bool {
        self.typ == 2
    }
    pub fn is_empty(&self) -> bool {
        self.typ == 0
    }

    pub fn path_buf(&self) -> PathBuf {
        let korean = encoding_from_whatwg_label("euc-kr").unwrap();
        let name = korean.decode(&self.name, DecoderTrap::Replace).unwrap();
        let p = PathBuf::from(name.trim_end_matches("\x00"));
        return p;
    }
}