/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align == 0 || (align & (align - 1)) != 0 {
        panic!("{} is not a power of 2", align);
    }
    (addr / align) * align
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    if align == 0 || (align & (align - 1)) != 0 {
        panic!("{} is not a power of 2", align);
    }
    if addr % align == 0 {
        return addr
    }
    ((addr / align) + 1) * align
}
