use core::fmt;
use alloc::string::String;
use alloc::string::ToString;
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
    pub first_cluster: Cluster,
    pub metadata: Metadata,
}

pub struct DirIter<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub raw_entries: Vec<VFatDirEntry>,
    pub pos: usize,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatRegularDirEntry {
    file_name: [u8; 8],
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


impl VFatRegularDirEntry {
    pub fn first_cluster(&self) -> Cluster {
        Cluster::from(self.low_bits_cluster_number as u32 | (self.high_bits_cluster_number as u32) << 16)
    }

    pub fn make_metadata(&self, name: String) -> Metadata {
        Metadata {
            attributes: self.attributes,
            created_ts: Timestamp { date: self.creation_date, time: self.creation_time },
            accessed_ts: Timestamp { date: self.last_accessed_date, time: Time(0) },
            modified_ts: Timestamp { date: self.last_modification_date, time: self.last_modification_time },
            name,
            size: self.file_size,
        }
    }

    pub fn make_regular_filename(&self) -> String {
        let file_name_len = self
            .file_name
            .iter()
            .position(|&b| b == 0x00 || b == 0x20)
            .unwrap_or(self.file_name.len());
        let file_ext_len = self
            .extension
            .iter()
            .position(|&b| b == 0x00 || b == 0x20)
            .unwrap_or(self.extension.len());
        let mut name = String::from_utf8_lossy(&self.file_name[..file_name_len]).to_string();
        if file_ext_len != 0 {
            let ext = String::from_utf8_lossy(&self.extension[..file_ext_len]).to_string();
            name += ".";
            name += &ext;
        }
        name
    }
}

impl fmt::Display for VFatRegularDirEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file_name: {} \n", self.make_regular_filename())?;
        write!(f, "low_bits_cluster_number: {:?} \n", &{ self.low_bits_cluster_number })?;
        write!(f, "high_bits_cluster_number: {:?} \n", &{ self.high_bits_cluster_number })
    }
}


#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    sequence_number: u8,
    name_characters_0: [u16; 5],
    attributes: Attributes,
    entry_type: u8,
    checksum: u8,
    name_characters_1: [u16; 6],
    empty: u16,
    name_characters_2: [u16; 2],
}

const_assert_size!(VFatLfnDirEntry, 32);

impl fmt::Display for VFatLfnDirEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut name_u16 = [0xffffu16; LFN_ENTRY_LEN];
        name_u16[0..5].copy_from_slice(&{ self.name_characters_0 });
        name_u16[5..11].copy_from_slice(&{ self.name_characters_1 });
        name_u16[11..13].copy_from_slice(&{ self.name_characters_2 });
        let name = String::from_utf16_lossy(&name_u16[..]);

        write!(f, "file_name: {}, ", name)?;
        write!(f, "sequence_number: {:X}", self.sequence_number)
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatUnknownDirEntry {
    id: u8,
    reserved0: [u8; 10],
    attributes: Attributes,
    reserved1: [u8; 20],
}

const_assert_size!(VFatUnknownDirEntry, 32);
#[derive(Copy, Clone)]
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
        use crate::traits::Dir;
        use crate::traits::Entry;
        let name = name
            .as_ref()
            .to_str()
            .ok_or(newioerr!(InvalidInput, "name is not utf-8"))?;
        self.entries()?
            .find(|e| e.name().eq_ignore_ascii_case(name))
            .ok_or(newioerr!(NotFound, "file name not found"))
    }
}

const MAX_LFN_ENTRIES: usize = 0x14;
const LFN_ENTRY_LEN: usize = 13;

fn handle_lfn_entry(lfn_entry: VFatLfnDirEntry, name_u16: &mut [u16]) {
    let seq_num = ((lfn_entry.sequence_number & 0x1f) - 1) as usize;
    assert!(seq_num < MAX_LFN_ENTRIES);
    let raw_name =
        &mut name_u16[seq_num * LFN_ENTRY_LEN..seq_num * LFN_ENTRY_LEN + LFN_ENTRY_LEN];
    raw_name[0..5].copy_from_slice(&{ lfn_entry.name_characters_0 });
    raw_name[5..11].copy_from_slice(&{ lfn_entry.name_characters_1 });
    raw_name[11..13].copy_from_slice(&{ lfn_entry.name_characters_2 });
}


impl<HANDLE: VFatHandle> Iterator for DirIter<HANDLE> {
    type Item = Entry<HANDLE>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut value: Option<Self::Item> = None;
        let mut name_u16 = [0xffffu16; MAX_LFN_ENTRIES * LFN_ENTRY_LEN];
        let mut encountered_lfn = false;

        for raw in self.raw_entries[self.pos..].into_iter() {
            self.pos += 1;
            let unknown_entry = unsafe { raw.unknown };
            match unknown_entry.id {
                0x00 => return None,
                0xE5 => { continue; }
                _ => {}
            }

            if unknown_entry.attributes.lfn() {
                encountered_lfn = true;
                let lfn_entry = unsafe { raw.long_filename };
                handle_lfn_entry(lfn_entry, &mut name_u16);
                continue;
            }

            let regular_entry = unsafe { raw.regular };
            let name = if encountered_lfn {
                let name_len = name_u16
                    .iter()
                    .position(|&b| b == 0x0000 || b == 0xffff)
                    .unwrap_or(name_u16.len());
                String::from_utf16_lossy(&name_u16[..name_len])
            } else {
                regular_entry.make_regular_filename()
            };

            let first_cluster = regular_entry.first_cluster();
            let metadata = regular_entry.make_metadata(name);

            let the_value = if regular_entry.attributes.directory() {
                Entry::Dir(Dir {
                    vfat: self.vfat.clone(),
                    first_cluster: first_cluster,
                    metadata,
                })
            } else {
                Entry::File(File {
                    vfat: self.vfat.clone(),
                    first_cluster: first_cluster,
                    metadata,
                    pos: 0,
                })
            };

            value = Some(the_value);
            break;
        }
        value
    }
}

impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    type Entry = Entry<HANDLE>;
    type Iter = DirIter<HANDLE>;

    fn entries(&self) -> io::Result<Self::Iter> {
        let mut data = Vec::new();
        self.vfat.lock(|vfat| -> io::Result<()> {
            vfat.read_chain(self.first_cluster, &mut data)?;
            Ok(())
        })?;

        Ok(DirIter {
            vfat: self.vfat.clone(),
            raw_entries: unsafe { data.cast() },
            pos: 0,
        })
    }
}
