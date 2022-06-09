use crate::traits;
use crate::vfat;
use crate::vfat::{Dir, File, Metadata, VFatHandle};
use core::fmt;

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    File(File<HANDLE>),
    Dir(Dir<HANDLE>),
}

// TODO: Implement any useful helper methods on `Entry`.

impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        "dummy"
    }

    fn metadata(&self) -> &Self::Metadata {
        unimplemented!()
    }

    fn as_file(&self) -> Option<&<Self as traits::Entry>::File> {
        match self {
            Entry::File(x) => Some(x),
            Entry::Dir(x) => None
        }
    }

    fn as_dir(&self) -> Option<&<Self as traits::Entry>::Dir> {
        match self {
            Entry::Dir(x) => Some(x),
            Entry::File(x) => None
        }
    }

    fn into_file(self) -> Option<<Self as traits::Entry>::File> {
        match self {
            Entry::File(x) => Some(x),
            Entry::Dir(x) => None
        }
    }

    fn into_dir(self) -> Option<<Self as traits::Entry>::Dir> {
        match self {
            Entry::Dir(x) => Some(x),
            Entry::File(x) => None
        }
    }

    fn is_file(&self) -> bool {
        match self {
            Entry::Dir(x) => false,
            Entry::File(x) => true
        }
    }

    fn is_dir(&self) -> bool {
        match self {
            Entry::Dir(x) => true,
            Entry::File(x) => false
        }
    }
}
