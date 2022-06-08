use core::fmt;
use core::fmt::Formatter;
use shim::const_assert_size;
use shim::io;
use core::convert::TryInto;

use crate::traits::BlockDevice;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    junk: [u8; 3],
}

impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Junk")?;
        Ok(())
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
pub struct PartitionEntry {
    boot: u8,
    starting_chs: CHS,
    partition_type: u8,
    ending_chs: CHS,
    relative_sector: u32,
    total_sectors_in_partition: u32,
}

impl fmt::Debug for PartitionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Boot: {}", self.boot);
        write!(f, "Starting CHS: ");
        self.starting_chs.fmt(f)?;
        write!(f, "Partition Type: {}", self.partition_type);
        write!(f, "Ending CHS: ");
        self.ending_chs.fmt(f)?;
        write!(f, "Total Sectors: {}", self.total_sectors_in_partition);
        Ok(())
    }
}

const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436],
    disk_id_1: [u8; 10],
    partition_table: [PartitionEntry; 4],
    signature: u16,
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.disk_id_1.fmt(f)?;
        for (i, table) in mbr.partition_table.iter().enumerate() {
            write!(f, "Partition {}: {}", i, table);
        }
        write!(f, "Signature: {}", self.signature);
        Ok(())
    }
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut buf = [0u8; 512]; // MBR is always 512
        device.read_sector(0, &mut buf);
        let mbr = unsafe { *{ buf.as_ptr() as *const MasterBootRecord } };
        if mbr.signature != 0x55AA {
            return Err(BadSignature)
        }
        for (i, table) in mbr.partition_table.iter().enumerate() {
            if table.boot != 0x0 && table.boot != 0x80 {
                return Err(UnknownBootIndicator(i.try_into().unwrap()))
            }
        }
        Ok(mbr)
    }
}
