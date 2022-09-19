use core::iter::{Chain, FlatMap};
use core::ops::{Deref, DerefMut};
use core::slice::Iter;

use alloc::boxed::Box;
use alloc::{fmt, vec};
use core::alloc::{GlobalAlloc, Layout};
use core::slice;
use crate::{allocator, kprintln};
use crate::param::*;
use crate::vm::{PhysicalAddr, VirtualAddr};
use crate::ALLOCATOR;

use aarch64::vmsa::*;
use shim::const_assert_size;

pub const VALID: u64 = 1;
pub const L3_ENTRY_TYPE: u64 = 1 << 1;
pub const INNER_SHAREABLE: u64 = 0b11 << 8;

pub const L3_INDEX_MASK: u64 = 0b1111111111111 << 16;
pub const L2_INDEX_MASK: u64 = 0b1111111111111 << 29;

pub const L3_ADDRESS_MASK: u64 = 0xFFFFFFFF << 16;


#[repr(C)]
pub struct Page([u8; PAGE_SIZE]);
const_assert_size!(Page, PAGE_SIZE);

impl Page {
    pub const SIZE: usize = PAGE_SIZE;
    pub const ALIGN: usize = PAGE_SIZE;

    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(Self::SIZE, Self::ALIGN) }
    }
}

#[repr(C)]
//#[repr(align(65536))]
pub struct L2PageTable {
    pub entries: [RawL2Entry; 8192],
}
const_assert_size!(L2PageTable, PAGE_SIZE);

impl L2PageTable {
    /// Returns a new `L2PageTable`
    fn new() -> L2PageTable {
        L2PageTable {
            entries: [RawL2Entry::new(0); 8192]
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        PhysicalAddr::from(&self.entries as *const RawL2Entry as u64)
    }
}

#[derive(Copy, Clone)]
pub struct L3Entry(RawL3Entry);

impl L3Entry {
    /// Returns a new `L3Entry`.
    fn new(v: u64) -> L3Entry {
        L3Entry(RawL3Entry::new(v))
    }

    /// Returns `true` if the L3Entry is valid and `false` otherwise.
    fn is_valid(&self) -> bool {
        self.0.get_value(RawL3Entry::VALID) > 0
    }

    /// Extracts `ADDR` field of the L3Entry and returns as a `PhysicalAddr`
    /// if valid. Otherwise, return `None`.
    fn get_page_addr(&self) -> Option<PhysicalAddr> {
        if self.is_valid() {
            return Some(PhysicalAddr::from(self.0.get_masked(RawL3Entry::ADDR)))
        }
        None
    }
}

#[repr(C)]
//#[repr(align(65536))]
pub struct L3PageTable {
    pub entries: [L3Entry; 8192],
}
const_assert_size!(L3PageTable, PAGE_SIZE);

impl L3PageTable {
    /// Returns a new `L3PageTable`.
    fn new() -> L3PageTable {
        L3PageTable {
            entries: [L3Entry::new(0); 8192]
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        PhysicalAddr::from(&self.entries as *const L3Entry as u64)
    }
}

#[repr(C)]
//#[repr(align(65536))]
pub struct PageTable {
    pub l2: L2PageTable,
    pub l3: [L3PageTable; 2],
}

impl PageTable {
    /// Returns a new `Box` containing `PageTable`.
    /// Entries in L2PageTable should be initialized properly before return.
    pub(crate) fn new(perm: u64) -> Box<PageTable> {
        let mut pt = PageTable {
            l2: L2PageTable::new(),
            l3: [L3PageTable::new(), L3PageTable::new()],
        };
        kprintln!("Initializing PageTable");
        for i in 0..2 {
            let addr = pt.l3[i].as_ptr();
            let mut entry = &mut pt.l2.entries[i];
            entry.set_masked(addr.as_u64(), RawL2Entry::ADDR);
            entry.set_bit(RawL2Entry::AF);
            entry.set_value(EntrySh::ISh, RawL2Entry::SH);
            entry.set_value(perm, RawL2Entry::AP);
            entry.set_value(EntryAttr::Mem, RawL2Entry::ATTR);
            entry.set_value(EntryType::Table, RawL2Entry::TYPE);
            entry.set_bit(RawL2Entry::VALID);
        }
        kprintln!("Finished populating the L2 table");
        Box::new(pt)
    }

    /// Returns the (L2index, L3index) extracted from the given virtual address.
    /// Since we are only supporting 4GB virtual memory in this system, L2index
    /// should be smaller than 8.
    ///
    /// # Panics
    ///
    /// Panics if the virtual address is not properly aligned to page size.
    /// Panics if extracted L2index exceeds the number of L3PageTable.
    fn locate(&self, va: VirtualAddr) -> (usize, usize) {
        let raw_address = va.as_u64();
        let l2index = (raw_address & L2_INDEX_MASK) >> 29;
        let l3index = (raw_address & L3_INDEX_MASK) >> 16;

        if l2index > 2 {
            panic!("l2_index > 8: {}", l2index);
        }

        let first_bits = raw_address & 0xFFFF;
        if first_bits != 0 {
            panic!("virtual address is not aligned 0x{:x}", raw_address);
        }

        return (l2index as usize, l3index as usize);
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is valid.
    /// Otherwise, `false` is returned.
    pub fn is_valid(&self, va: VirtualAddr) -> bool {
        let (l2index, l3index) = self.locate(va);

        let l3_entry = self.l3[l2index].entries[l3index];

        l3_entry.is_valid()
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is invalid.
    /// Otherwise, `true` is returned.
    pub fn is_invalid(&self, va: VirtualAddr) -> bool {
        !self.is_valid(va)
    }

    /// Set the given RawL3Entry `entry` to the L3Entry indicated by the given virtual
    /// address.
    pub fn set_entry(&mut self, va: VirtualAddr, entry: RawL3Entry) -> &mut Self {
        let (l2index, l3index) = self.locate(va);

        self.l3[l2index].entries[l3index] = L3Entry(entry);
        self
    }

    /// Returns a base address of the pagetable. The returned `PhysicalAddr` value
    /// will point the start address of the L2PageTable.
    pub fn get_baddr(&self) -> PhysicalAddr {
        self.l2.as_ptr()
    }
}

impl<'a> IntoIterator for &'a PageTable {
    type Item = &'a L3Entry;
    type IntoIter = FlatMap<Iter<'a,L3PageTable>, Iter<'a, L3Entry>, fn(&L3PageTable) -> Iter<L3Entry>>;

    fn into_iter(self) -> Self::IntoIter {
        return self.l3.iter().flat_map(|l| l.entries.iter())
    }
}


pub struct KernPageTable(Box<PageTable>);

impl KernPageTable {
    /// Returns a new `KernPageTable`. `KernPageTable` should have a `Pagetable`
    /// created with `KERN_RW` permission.
    ///
    /// Set L3entry of ARM physical address starting at 0x00000000 for RAM and
    /// physical address range from `IO_BASE` to `IO_BASE_END` for peripherals.
    /// Each L3 entry should have correct value for lower attributes[10:0] as well
    /// as address[47:16]. Refer to the definition of `RawL3Entry` in `vmsa.rs` for
    /// more details.
    pub fn new() -> KernPageTable {
        kprintln!("Making a new KernPageTable");
        let start_address = 0x0;
        let (_, end_address) = allocator::memory_map().unwrap();
        kprintln!("Got memory map!");

        let mut page_table = PageTable::new(EntryPerm::KERN_RW);

        for addr in (start_address..end_address).step_by(PAGE_SIZE) {
            let va = VirtualAddr::from(addr);

            let mut entry = RawL3Entry::new(0);
            entry.set_masked(addr as u64, RawL3Entry::ADDR);
            entry.set_bit(RawL3Entry::AF);
            entry.set_value(EntryPerm::KERN_RW, RawL3Entry::AP);
            entry.set_value(EntryType::Table, RawL3Entry::TYPE);
            entry.set_bit(RawL3Entry::VALID);

            if addr >= IO_BASE && addr <= IO_BASE_END {
                entry.set_value(EntrySh::OSh, RawL3Entry::SH);
                entry.set_value(EntryAttr::Dev, RawL3Entry::ATTR);
            } else {
                entry.set_value(EntrySh::ISh, RawL3Entry::SH);
                entry.set_value(EntryAttr::Mem, RawL3Entry::ATTR);
            }

            page_table.set_entry(va, entry);
        }

        return KernPageTable(page_table);
    }
}

pub enum PagePerm {
    RW,
    RO,
    RWX,
}

pub struct UserPageTable(Box<PageTable>);

impl UserPageTable {
    /// Returns a new `UserPageTable` containing a `PageTable` created with
    /// `USER_RW` permission.
    pub fn new() -> UserPageTable {
        Self(PageTable::new(EntryPerm::USER_RW))
    }

    /// Allocates a page and set an L3 entry translates given virtual address to the
    /// physical address of the allocated page. Returns the allocated page.
    ///
    /// # Panics
    /// Panics if the virtual address is lower than `USER_IMG_BASE`.
    /// Panics if the virtual address has already been allocated.
    /// Panics if allocator fails to allocate a page.
    ///
    /// TODO. use Result<T> and make it failurable
    /// TODO. use perm properly
    pub fn alloc(&mut self, va: VirtualAddr, _perm: PagePerm) -> &mut [u8] {
        if va.as_usize() < USER_IMG_BASE {
            panic!("va < USER_IMG_BASE: 0x{:x}", va.as_u64());
        }
        let va_offset = va - VirtualAddr::from(USER_IMG_BASE);
        if self.0.is_valid(va_offset) {
            panic!("va already allocated: 0x{:x}", va.as_u64());
        }
        let addr = unsafe { ALLOCATOR.alloc(Page::layout()) as u64 };
        if addr == 0 {
            panic!("allocation failed");
        }

        let mut entry = RawL3Entry::new(0);
        entry.set_masked(addr as u64, RawL3Entry::ADDR);
        entry.set_bit(RawL3Entry::AF);
        entry.set_value(EntrySh::ISh, RawL3Entry::SH);
        entry.set_value(EntryPerm::USER_RW, RawL3Entry::AP);
        entry.set_value(EntryAttr::Mem, RawL3Entry::ATTR);
        entry.set_value(EntryType::Table, RawL3Entry::TYPE);
        entry.set_bit(RawL3Entry::VALID);
        self.0.set_entry(va_offset, entry);
        unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, PAGE_SIZE) }
    }
}

impl Deref for KernPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for UserPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KernPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for UserPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl Drop for UserPageTable {
    fn drop(&mut self) {
        for entry in self.0.into_iter() {
            if entry.is_valid() {
                let addr = entry.get_page_addr().unwrap();
                unsafe {
                    ALLOCATOR.dealloc(addr.as_u64() as *mut u8, Page::layout());
                }
            }
        }
    }
}

impl fmt::Debug for UserPageTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "UserPageTable {{")?;
        for entry in self.0.into_iter() {
            if entry.is_valid() {
                writeln!(f, "  0x{:08x}", entry.get_page_addr().unwrap().as_u64())?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}
