use core::mem::size_of;

use crate::config::USER_ASPACE_RANGE;

fn uaccess_ok(vaddr: usize, size: usize) -> bool {
    USER_ASPACE_RANGE.start <= vaddr && vaddr + size <= USER_ASPACE_RANGE.end
}

pub unsafe fn copy_from_user<T>(kdst: *mut T, usrc: *const T, len: usize) {
    assert!(uaccess_ok(usrc as usize, len * size_of::<T>()));
    kdst.copy_from_nonoverlapping(usrc, len);
}

pub unsafe fn copy_to_user<T>(udst: *mut T, ksrc: *const T, len: usize) {
    assert!(uaccess_ok(udst as usize, len * size_of::<T>()));
    udst.copy_from_nonoverlapping(ksrc, len);
}
