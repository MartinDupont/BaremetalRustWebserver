use core::fmt;

use alloc::string::String;
use core::fmt::{Debug, Formatter};

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

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
    creation_time_tenths: u8,
    pub creation_time: Time,
    pub creation_date: Date,
    pub last_accessed_date: Date,
    high_bits_cluster_number: u8,
    pub last_modification_time: Time,
    pub last_modification_date: Date,
    pub low_bits_cluster_number: u8,
    pub file_size: u32,
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
        self.attributes.0 == 0x01
    }

    fn hidden(&self) -> bool {
        self.attributes.0 == 0x02
    }

    fn created(&self) -> Self::Timestamp {
        Timestamp {
            date: self.creation_date,
            time: self.creation_time,
        }
    }

    fn accessed(&self) -> Self::Timestamp {
        Timestamp {
            date: self.last_accessed_date,
            time: Time(0),
        }
    }

    fn modified(&self) -> Self::Timestamp {
        Timestamp {
            date: self.last_modification_date,
            time: self.last_modification_time,
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} / {} / {}: {}:{}:{}", self.day(), self.month(), self.year(), self.hour(), self.minute(), self.second())?;
        Ok(())
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Readonly: {}", self.read_only())?;
        write!(f, "Hidden: {}", self.hidden())?;
        write!(f, "Created")?;
        self.created().fmt()?;
        write!(f, "Accessed")?;
        self.accessed().fmt()?;
        write!(f, "Modified")?;
        self.modified().fmt()?;

        Ok(())
    }
}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
