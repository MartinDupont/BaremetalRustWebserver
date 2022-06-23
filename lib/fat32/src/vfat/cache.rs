use alloc::boxed::Box;
use core::{cmp, fmt};

use shim::io;

use crate::traits::BlockDevice;

#[derive(Debug)]
pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// Number of sectors
    pub num_sectors: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct PartitionedDevice {
    device: Box<dyn BlockDevice>,
    partition: Partition,
}

impl PartitionedDevice {
    /// Creates a new `PartitionedDevice` that maps physical sectors to logical sectors inside of
    /// `partition`.
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
    pub fn new<T>(device: T, partition: Partition) -> PartitionedDevice
        where
            T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());
        assert!(partition.sector_size % device.sector_size() == 0);

        PartitionedDevice {
            device: Box::new(device),
            partition: partition,
        }
    }

    /// Returns the number of physical sectors that corresponds to
    /// one logical sector.
    fn factor(&self) -> u64 {
        self.partition.sector_size / self.device.sector_size()
    }

    /// Maps a user's request for a sector `virt` to the physical sector.
    /// Returns `None` if the virtual sector number is out of range.
    fn virtual_to_physical(&self, virt: u64) -> Option<u64> {
        if virt >= self.partition.num_sectors {
            return None;
        }

        let physical_offset = virt * self.factor();
        let physical_sector = self.partition.start + physical_offset;

        Some(physical_sector)
    }
}



impl BlockDevice for PartitionedDevice {
    fn sector_size(&self) -> u64 {
        self.partition.sector_size
    }

    fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> io::Result<usize> {
        let real_sector = self.virtual_to_physical(sector).ok_or(io::Error::new(io::ErrorKind::BrokenPipe, "Not a broken pipe. virtual address is wrong"))?;
        let physical_sector_size = self.device.sector_size() as usize;
        let mut read_bytes = 0;
        let n = self.factor();

        for i in 0..n as usize {
            let end = cmp::min((i + 1) * physical_sector_size, buf.len());

            let num = self.device.read_sector(
                real_sector + i as u64,
                &mut buf[i * physical_sector_size..end],
            )?;
            read_bytes += num;

            if end == buf.len(){
                break;
            }

        }

        Ok(read_bytes)
    }

    fn write_sector(&mut self, _sector: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }
}

impl fmt::Debug for PartitionedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PartitionedDevice")
            .field("device", &"<block device>")
            .field("partition", &self.partition)
            .finish()
    }
}

