use core::alloc::Layout;
use core::fmt;
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///
///   map_to_bin(size) -> k
///

const BINS_START_K: usize = 3;
pub const BINS_LEN: usize = 30;

pub struct Allocator {
    start: usize,
    end: usize,
    bins: [LinkedList; BINS_LEN],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Self {
            bins: [LinkedList::new(); BINS_LEN],
            start,
            end,
        }
    }
}

const fn bin_index_size(index: usize) -> usize {
    1 << (BINS_START_K + index)
}

pub fn get_bin_for_size(size: usize) -> Result<usize, ()> {
    for i in 0..BINS_LEN {
        if size <= bin_index_size(i) {
            return Ok(i);
        }
    }
    return Err(());
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let mut bin_number = get_bin_for_size(layout.size());
        if bin_number.is_err() {
            return core::ptr::null_mut()
        }
        let n = bin_number.unwrap();

        let bin_size = bin_index_size(n);

        for i in n..BINS_LEN {
            for node in self.bins[i].iter_mut() {
                let addr = node.value() as usize;
                let addr_align = align_up(addr, layout.align());
                if addr == addr_align {
                    node.pop();
                    // If we found a bigger bin than needed, move second unused half of the memory
                    // into the previous bin
                    if i > n {
                        let this_bin_size = bin_index_size(i);
                        self.bins[i - 1].push((addr + this_bin_size / 2) as *mut usize);
                    }
                    return addr as *mut u8;
                }
            }
        }

        // nothing available
        let start = align_up(self.start, layout.align());
        let (end, overflow) = start.overflowing_add(bin_size);
        if overflow || end > self.end {
            return core::ptr::null_mut();
        } else {
            self.start = end;
            start as *mut u8
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let bin_number = get_bin_for_size(layout.size())
            .expect("should never be deallocating something that could not have been allocated");
        self.bins[bin_number].push(ptr as *mut usize);
    }
}

use core::fmt::Debug;

impl Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Allocator {{ start: {}, end: {} }}\n",
            self.start, self.end
        )?;
        for (i, bin) in self.bins.iter().enumerate() {
            write!(f, "Bin {} [2^{}]: {:?}", i, BINS_START_K + i, bin)?;
        }
        Ok(())
    }
}
