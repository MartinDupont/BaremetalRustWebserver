use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::{Component, Path};

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{Attributes, BiosParameterBlock, CachedPartition, Metadata, Timestamp, Date, Time, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};
use crate::vfat::Error::NotFormatted;

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
        let mbr = MasterBootRecord::from(&mut device)?;

        let first_partition = mbr.partition_table[0];

        if first_partition.partition_type != 0xB && first_partition.partition_type != 0xC {
            return Err(NotFormatted);
        }

        let ebpb = BiosParameterBlock::from(&mut device, first_partition.relative_sector as u64)?;


        let partition = Partition {
            start: first_partition.relative_sector as u64,
            num_sectors: first_partition.total_sectors_in_partition as u64,
            sector_size: ebpb.bytes_per_sector as u64,
        };
        let cached_partition = CachedPartition::new(device, partition);

        let rootdir_cluster = Cluster::from(ebpb.cluster_number_of_root);

        let vfat = VFat {
            phantom: Default::default(),
            device: cached_partition,
            bytes_per_sector: ebpb.bytes_per_sector, // TODO: CHeck little endianness.
            sectors_per_cluster: ebpb.sectors_per_cluster,
            sectors_per_fat: ebpb.sectors_per_fat(),
            fat_start_sector: ebpb.number_reserved_sectors as u64,
            data_start_sector: ebpb.number_reserved_sectors as u64 + (ebpb.number_fats as u64 * ebpb.sectors_per_fat() as u64),
            rootdir_cluster: rootdir_cluster,
        };
        Ok(HANDLE::new(vfat))
    }

    fn get_sector_for_cluster(&self, cluster: Cluster) -> u64 {
        self.data_start_sector + (cluster.raw() as u64 * self.sectors_per_cluster as u64)
    }

    pub fn read_cluster(
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

    pub fn read_chain(
        &mut self,
        start: Cluster,
        buf: &mut Vec<u8>,
    ) -> io::Result<usize> {
        let mut fat = self.fat_entry(start)?;
        let mut cluster = start;
        let mut total: usize = 0;
        loop {
            let status = fat.status();
            if status == Status::Free || status == Status::Bad || status == Status::Reserved {
                return Err(io::Error::new(io::ErrorKind::Other, "Attempted to read VFAT partition which was reserved, bad, or free"));
            }
            // TODO: Refactor this to avoid the panic!() call.
            let mut array_buf = vec![0u8; self.bytes_per_sector as usize];
            let bytes_read = self.read_cluster(cluster, 0, &mut array_buf)?;
            total += bytes_read;
            buf.extend_from_slice(&array_buf);

            match status {
                Status::Eoc(x) => { break; }
                Status::Data(x) => {
                    fat = self.fat_entry(x)?;
                    cluster = x;
                }
                _ => panic!("This code should be unreachable")
            }
        }
        Ok(total)
    }

    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<FatEntry> {
        use core::mem::size_of;
        let fat_entries_per_sector = self.device.sector_size() as usize / size_of::<FatEntry>();
        let sector = self.fat_start_sector + cluster.raw() as u64 / (fat_entries_per_sector as u64);
        let offset = cluster.raw() as usize % (fat_entries_per_sector as usize);
        let offset_bytes = offset * size_of::<FatEntry>();
        let sector_data = self.device.get(sector)?;
        let mut bytes = [0; 4];
        bytes.copy_from_slice(&sector_data[offset_bytes..offset_bytes + 4]);
        println!("FAT start sector: {}", self.fat_start_sector);
        println!("Sector where my cluster is: {}", sector);
        println!("Sector data: {:?}", sector_data);


        Ok(FatEntry(u32::from_le_bytes(bytes)))

    }
}

fn make_root_dir_metadata() -> Metadata {
    Metadata {
        attributes: Attributes(0),
        created_ts: Timestamp { date: Date(0), time: Time(0) },
        accessed_ts: Timestamp { date: Date(0), time: Time(0) },
        modified_ts: Timestamp { date: Date(0), time: Time(0) },
        name: String::from("/"),
        size: 0
    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        let mut dir = Dir {
            vfat: self.clone(),
            first_cluster: self.lock(|vfat| vfat.rootdir_cluster),
            metadata: make_root_dir_metadata(),
        };

        self.lock(|vfat| { println!("Rootdir cluster: {:?}", vfat.rootdir_cluster); });

        let mut file: Option<File<HANDLE>> = None;

        for component in path.as_ref().components() {
            if component == Component::RootDir {
                continue;
            }
            if file.is_some() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Not a directory"));
            }
            let found = dir.find(component)?;

            match found {
                Entry::Dir(x) => { dir = x }
                Entry::File(x) => { file = Some(x); }
            }
        }
        Ok(Entry::Dir(dir))
    }
}
