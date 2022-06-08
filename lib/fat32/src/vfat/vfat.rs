use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::Path;

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};

/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    device: CachedPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
        where
            T: BlockDevice + 'static,
    {
        let mbr = MasterBootRecord::from(device)?;
        let ebpb = BiosParameterBlock::from(device, 1)?;

        let first_partition = mbr.partition_table[0];

        let partition = Partition {
            start: first_partition.relative_sector as u64,
            num_sectors: first_partition.total_sectors_in_partition as u64,
            sector_size: ebpb.bytes_per_sector as u64 * first_partition.total_sectors_in_partition as u64,
        };
        let cached_partition = CachedPartition::new(device, partition);

        let rootdir_cluster = Cluster::from(ebpb.cluster_number_of_root);

        let vfat = VFat {
            phantom: Default::default(),
            device: cached_partition,
            bytes_per_sector: ebpb.bytes_per_sector, // TODO: CHeck little endianness.
            sectors_per_cluster: ebpb.sectors_per_cluster,
            sectors_per_fat: ebpb.sectors_per_fat,
            fat_start_sector: 1 + ebpb.number_reserved_sectors as u64,
            data_start_sector: 1 + ebpb.number_reserved_sectors as u64 + (ebpb.number_fats as u64 * ebpb.number_sectors_per_fat as u64),
            rootdir_cluster: rootdir_cluster,
        };
        Ok(HANDLE::new(vfat))
    }

    fn get_sector_for_cluster(&self, cluster: Cluster) -> u64 {
        self.data_start_sector + (cluster.raw() as u64 * self.sectors_per_cluster as u64)
    }

    fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        let sector = self.get_sector_for_cluster(cluster);
        let sector_data = self.device.get(sector)?;

        buf.copy_from_slice(&sector_data[offset..]);
        Ok(buf.len())
    }

    //* A method to read all of the clusters chained from a starting cluster into a vector.

    // fn read_chain(
    //     &mut self,
    //     start: Cluster,
    //     buf: &mut Vec<u8>,
    // ) -> io::Result<usize>;

    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let fat_entries_per_sector = self.device.sector_size() as usize / 4;
        let sector = self.fat_start_sector + cluster.raw() as u64 / (fat_entries_per_sector as u64);
        let offset = cluster.raw() as usize % (fat_entries_per_sector as usize);
        let offset_bytes = offset * 4;
        let sector_data = self.device.get(sector)?;
        let mut bytes = [0; 4];

        bytes.copy_from_slice(&sector_data[offset_bytes..offset_bytes + 4]);
        Ok(&FatEntry(u32::from_le_bytes(bytes)))
    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = crate::traits::Dummy;
    type Dir = crate::traits::Dummy;
    type Entry = crate::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
    }
}
