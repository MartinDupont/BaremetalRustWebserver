use core::fmt;
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;
use crate::vfat::Error::{NotFormatted, BadSignature};

#[repr(C, packed)]
pub struct BiosParameterBlock {
    jump_short_noop: [u8; 3],
    oem_identifier: u64,
    bytes_per_sector: u16,
    // It's little endian!!
    number_reserved_sectors: u16,
    number_fats: u16,
    max_number_directory_entries: u16,
    total_logical_sectors_16: u16,
    media_descriptor_type: u8,
    number_sectors_per_fat: u16,
    number_sectors_per_track: u16,
    number_heads: u16,
    number_hidden_sectors: u32,
    total_logical_sectors_32: u32,
    sectors_per_fat: u32,
    flags: u16,
    fat_version_number: u16,
    cluster_number_of_root: u32,
    sector_number_of_fs_info: u16,
    sector_number_backup_boot: u16,
    __reserved: [u8; 12],
    // when the volume is formatted these bytes should be zero
    drive_number: u8,
    __reserved_flags_windows_nt: u8,
    signature: u8,
    // 1 Signature (should be 0x28 or 0x29).
    volume_id_serial_number: u32,
    // Used for tracking volumes between computers. You can ignore this if you want.
    volume_label_string: [u8; 11],
    system_identifier_string: u64,
    // Always "FAT32   ". The spec says never to trust the tents of this string for any use.
    boot_code: [u8; 420],
    bootable_partition_signature: u16, //       2 0xAA55
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut buf = [0u8; 512]; // EBPB is always 512
        device.read_sector(sector, &mut buf);
        let ebpb = unsafe { *{ buf.as_ptr() as *const BiosParameterBlock } };
        if ebpb.bootable_partition_signature != 0xAA55 {
            return Err(BadSignature)
        }
        if ebpb.signature != 0x28 && ebpb.signature != 0x29 {
            return Err(BadSignature)
        }

        if ebpb.__reserved != [0u8; 12] {
            return Err(NotFormatted)
        }
        Ok(ebpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!("BiosParameterBlock::fmt()")
    }
}
