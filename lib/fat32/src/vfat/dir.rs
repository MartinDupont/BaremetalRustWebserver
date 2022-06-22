use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

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
}


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
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

impl<HANDLE: VFatHandle> Iterator for DirIter<HANDLE> {
    type Item = Entry<HANDLE>;

    // The first byte of an entry (whether regular or LFN) is also known as the ID.
    // ID of 0x00. Indicates the end of the directory.
    // ID of 0xE5: Marks an unused/deleted entry.
    // All other IDs make up part of the file’s name or LFN sequence number.
    // The byte at offset 11 determines whether the entry is a regular entry or an LFN entry.
    // Value of 0x0F: entry is an LFN entry.
    // All other values: entry is a regular entry

    fn next(&mut self) -> Option<Self::Item> {
        let mut name = String::new();

        let mut value: Option<Self::Item> = None;

        for raw in self.raw_entries[self.pos..].into_iter() {
            let unknown_entry = unsafe { raw.unknown };
            match unknown_entry.id {
                0x00 => return None,
                0xE5 => { continue; }
                _ => {}
            }

            if unknown_entry.attributes.lfn() {
                continue
            }

            let regular_entry = unsafe { raw.regular };
            println!("===========================================================================");
            println!("Entry: {:?}", regular_entry);
            //name = String::from_utf8_lossy(&regular_entry.file_name[..] ).to_string();
            name = String::from("filename.txt");

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
                })
            };

            self.pos += 1;
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

        // need to conver my data into ...... something useful. I need to parse the VFATClasses here.
        // I have already followed a chain of clusters. So I have parsed the SHIT.
        // But how does this cluster chain relate to directories? How do I tell if something is a directory?

        // How can a directory contain both directories and files?????

        // Where do I convert from raw classes into the higher classes?

        // I need to ask metadata.attributes to find out whether the thing I need is a subdirectory or not.

        // If I know I have a directory. How do I get its children??

        // Can both Directories and files have long file names????

        // I still don't think I've LOADED the data yet......


        Ok(DirIter {
            vfat: self.vfat.clone(),
            raw_entries: unsafe { data.cast() },
            pos: 0
        })
    }
}
