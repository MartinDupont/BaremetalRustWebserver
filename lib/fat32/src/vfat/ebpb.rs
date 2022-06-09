use core::fmt;
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;
use crate::vfat::Error::{NotFormatted, BadSignature};

// TODO: Check endianness of sectors per cluster. It's little endian!!
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct BiosParameterBlock {
    pub jump_short_noop: [u8; 3],
    pub oem_identifier: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub number_reserved_sectors: u16,
    pub number_fats: u8,
    pub max_number_directory_entries: u16,
    pub total_logical_sectors_16: u16,
    pub media_descriptor_type: u8,
    pub number_sectors_per_fat: u16,
    pub number_sectors_per_track: u16,
    pub number_heads: u16,
    pub number_hidden_sectors: u32,
    pub total_logical_sectors_32: u32,
    pub sectors_per_fat: u32,
    pub flags: u16,
    pub fat_version_number: u16,
    pub cluster_number_of_root: u32,
    pub sector_number_of_fs_info: u16,
    pub sector_number_backup_boot: u16,
    pub __reserved: [u8; 12],
    // when the volume is formatted these bytes should be zero
    pub drive_number: u8,
    pub __reserved_flags_windows_nt: u8,
    pub signature: u8,
    // 1 Signature (should be 0x28 or 0x29).
    pub volume_id_serial_number: u32,
    // Used for tracking volumes between computers. You can ignore this if you want.
    pub volume_label_string: [u8; 11],
    pub system_identifier_string: u64,
    // Always "FAT32   ". The spec says never to trust the tents of this string for any use.
    pub boot_code: [u8; 420],
    pub bootable_partition_signature: u16, //       2 0xAA55
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
        device.read_sector(sector, &mut buf).map_err(|error| { Error::Io(error) })?;
        let ebpb = unsafe { *{ buf.as_ptr() as *const BiosParameterBlock } };

        if ebpb.bootable_partition_signature != 0xAA55 {
            println!("EBPB bootable not 0xAAFF, is instead {:#08x}", ebpb.bootable_partition_signature);
            return Err(BadSignature);
        }
        // if ebpb.signature != 0x28 && ebpb.signature != 0x29 {
        // println!("EBPB sig not 0x28 or 0x29, is instead {:#08x}", ebpb.signature);
        //     return Err(BadSignature);
        // }

        if ebpb.__reserved != [0u8; 12] {
            return Err(NotFormatted);
        }
        Ok(ebpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("jump_short_noop", &self.jump_short_noop)
            .field("oem_identifier", &self.oem_identifier)
            .field("bytes_per_sector", &self.bytes_per_sector)
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("number_reserved_sectors", &self.number_reserved_sectors)
            .field("number_fats", &self.number_fats)
            .field("max_number_directory_entries", &self.max_number_directory_entries)
            .field("total_logical_sectors_16", &self.total_logical_sectors_16)
            .field("media_descriptor_type", &self.media_descriptor_type)
            .field("number_sectors_per_fat", &self.number_sectors_per_fat)
            .field("number_sectors_per_track", &self.number_sectors_per_track)
            .field("number_heads", &self.number_heads)
            .field("number_hidden_sectors", &self.number_hidden_sectors)
            .field("total_logical_sectors_32", &self.total_logical_sectors_32)
            .field("sectors_per_fat", &self.sectors_per_fat)
            .field("flags", &self.flags)
            .field("oem_id", &self.fat_version_number)
            .field("cluster_number_of_root", &self.cluster_number_of_root)
            .field("sector_number_of_fs_info", &self.sector_number_of_fs_info)
            .field("sector_number_backup_boot", &self.sector_number_backup_boot)
            .field("__reserved", &self.__reserved)
            .field("drive_number", &self.drive_number)
            .field("__reserved_flags_windows_nt", &self.__reserved_flags_windows_nt)
            .field("signature", &self.signature)
            .field("volume_id_serial_number", &self.volume_id_serial_number)
            .field("volume_label_string", &self.volume_label_string)
            .field("system_identifier_string", &self.system_identifier_string)
            //.field("boot_code", &self.boot_code)
            .finish()
    }
}
