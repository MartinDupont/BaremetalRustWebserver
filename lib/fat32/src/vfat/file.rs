use shim::io::{self, SeekFrom};
use shim::{ioerr};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};

use alloc::vec::Vec;

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub metadata: Metadata,
    pub first_cluster: Cluster,
    pub pos: usize,
}

impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn size(&self) -> u64 {
        self.metadata.size as u64
    }
}

impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

        if self.pos as u32 == self.metadata.size {
            return Ok(0);
        } else if self.pos  as u32 >= self.metadata.size {
            return ioerr!(InvalidInput, "read past the end of file");
        }

        let mut data = Vec::new();
        self.vfat.lock(|vfat| -> io::Result<()> {
            vfat.read_chain(self.first_cluster, &mut data)?;
            Ok(())
        })?;

        let a = core::cmp::min(self.metadata.size as usize, data.len());
        let len = core::cmp::min(buf.len(), a - self.pos);
        buf[..len]
            .copy_from_slice(&data[self.pos..(self.pos + len)]);

        self.pos += len;

        Ok(len)
    }
}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Ok(0)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

}


impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        unimplemented!("File::seek()")
    }
}
