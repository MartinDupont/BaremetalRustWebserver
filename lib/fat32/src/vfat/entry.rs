use crate::traits;
use crate::vfat;
use crate::vfat::{Dir, File, Metadata, VFatHandle};
use core::fmt;

#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    File(File<HANDLE>),
    Dir(Dir<HANDLE>),
}

impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        match self {
            Entry::File(x) => &x.metadata.name,
            Entry::Dir(x) => &x.metadata.name
        }
    }

    fn metadata(&self) -> &Self::Metadata {
        match self {
            Entry::File(x) => &x.metadata,
            Entry::Dir(x) => &x.metadata
        }
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
