use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use hashbrown::{hash_map::Entry, HashMap};
use shim::io;

use crate::traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// Number of sectors
    pub num_sectors: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedDevice {
    device: Box<dyn BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
}

impl CachedDevice {
    /// Creates a new `CachedPartition` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `0` will be
    /// translated to physical sector `partition.start`. Virtual sectors of
    /// sector number `[0, num_sectors)` are accessible.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T) -> CachedDevice
        where
            T: BlockDevice + 'static,
    {

        CachedDevice {
            device: Box::new(device),
            cache: HashMap::new(),
        }
    }


    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        let mut cache_entry = match self.cache.entry(sector) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let mut sector_data = vec![0; self.device.sector_size() as usize];
                self.device.read_sector(sector, &mut sector_data)?;
                entry.insert(CacheEntry {
                    data: sector_data,
                    dirty: false,
                })
            }
        };
        cache_entry.dirty = true;
        Ok(&mut cache_entry.data)
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        let mut cache_entry = match self.cache.entry(sector) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let mut sector_data = vec![0; self.device.sector_size() as usize];
                self.device.read_sector(sector, &mut sector_data)?;
                entry.insert(CacheEntry {
                    data: sector_data,
                    dirty: false,
                })
            }
        };
        cache_entry.dirty = true;
        Ok(&cache_entry.data)
    }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`. The `read_sector` and
// `write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedDevice {
    fn sector_size(&self) -> u64 {
        self.device.sector_size()
    }

    fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> io::Result<usize> {
        let data = self.get(sector)?;
        buf.clone_from_slice(data);
        Ok(data.len())
    }

    fn write_sector(&mut self, sector: u64, buf: &[u8]) -> io::Result<usize> {
        let cache_entry = CacheEntry {
            data: buf.to_vec(),
            dirty: true,
        };
        // TODO: Do I care if the entry existed before or not? THe comment isn't clear
        self.cache.insert(sector, cache_entry);
        Ok(buf.len())
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedPartition")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
