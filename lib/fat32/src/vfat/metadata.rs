use core::fmt;

use alloc::string::String;
use core::fmt::{Debug, Formatter};

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(pub u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(pub u16);

const ATTR_READ_ONLY: u8 = 1 << 0;
const ATTR_HIDDEN: u8 = 1 << 1;
const ATTR_SYSTEM: u8 = 1 << 2;
const ATTR_VOLUME_ID: u8 = 1 << 3;
const ATTR_DIRECTORY: u8 = 1 << 4;
const ATTR_ARCHIVE: u8 = 1 << 5;
const ATTR_LFN: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(pub u8);

const ROOTDIR_ATTRIBUTES: Attributes = Attributes(ATTR_DIRECTORY);

impl Attributes {
    pub fn raw(&self) -> u8 {
        self.0
    }
    pub fn read_only(&self) -> bool {
        self.0 & ATTR_READ_ONLY != 0
    }
    pub fn hidden(&self) -> bool {
        self.0 & ATTR_HIDDEN != 0
    }
    pub fn system(&self) -> bool {
        self.0 & ATTR_SYSTEM != 0
    }
    pub fn volume_id(&self) -> bool {
        self.0 & ATTR_VOLUME_ID != 0
    }
    pub fn directory(&self) -> bool {
        self.0 & ATTR_DIRECTORY != 0
    }
    pub fn archive(&self) -> bool {
        self.0 & ATTR_ARCHIVE != 0
    }
    pub fn lfn(&self) -> bool {
        self.0 & ATTR_LFN == ATTR_LFN
    }
}
/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub attributes: Attributes,
    pub created_ts: Timestamp,
    pub accessed_ts: Timestamp,
    pub modified_ts: Timestamp,
    pub name: String,
}

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        (self.date.0 >> 9) as usize
    }

    fn month(&self) -> u8 {
        let mask = 0b0000000011100000;
        ((self.date.0 & mask) >> 5) as u8
    }

    fn day(&self) -> u8 {
        let mask = 0b0000000000011111;
        (self.date.0 & mask) as u8
    }

    fn hour(&self) -> u8 {
        (self.date.0 >> 11) as u8
    }

    fn minute(&self) -> u8 {
        let mask = 0b0000011111100000;
        ((self.date.0 & mask) >> 5) as u8
    }

    fn second(&self) -> u8 {
        let mask = 0b0000000000011111;
        (self.time.0 & mask) as u8 * 2
    }
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        self.attributes.read_only()
    }

    fn hidden(&self) -> bool {
        self.attributes.hidden()
    }

    fn created(&self) -> Self::Timestamp {
        self.created_ts
    }

    fn accessed(&self) -> Self::Timestamp {
        self.accessed_ts
    }

    fn modified(&self) -> Self::Timestamp {
        self.modified_ts
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use crate::traits::Timestamp;
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second()
        )
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use traits::Metadata;

        f.debug_struct("Metadata")
            .field("read_only", &self.read_only())
            .field("hidden", &self.hidden())
            .field("created", &self.created())
            .field("accessed", &self.accessed())
            .field("modified", &self.modified())
            .finish()
    }
}
