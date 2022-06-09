use alloc::string::String;
use alloc::vec::Vec;

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;

use crate::traits;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};

#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    // FIXME: Fill me in.
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    file_name: u64,
    extension: [u8; 3],
    pub attributes: Attributes,
    __reserved: u8,
    creation_time_tenths: u8,
    pub creation_time: Time,
    pub creation_date: Date,
    pub last_accessed_date: Date,
    high_bits_cluster_number: u16,
    pub last_modification_time: Time,
    pub last_modification_date: Date,
    pub low_bits_cluster_number: u16,
    pub file_size: u32,
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    sequence_number: u8,
    name_characters: [u8; 10],
    attributes: Attributes,
    entry_type: u8,
    checksum: u8,
    name_characters_2: [u8;12],
    empty: u16,
    name_characters_3: u32,
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    who_fucking_knows: [u8; 32],
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        unimplemented!("Dir::find()")
    }
}

impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    type Entry = traits::Dummy;
    type Iter = traits::Dummy;

    fn entries(&self) -> io::Result<Self::Iter> {
        unimplemented!()

    }
    // FIXME: Implement `trait::Dir` for `Dir`.
}
