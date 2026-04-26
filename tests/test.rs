#![no_std]

use core::ffi::c_char;

unsafe extern "C" {
    unsafe fn puts(str: *const c_char) -> i32;
}

#[unsafe(no_mangle)]
pub fn test() {
    let str = b"hello world :3\0";
    unsafe {
        puts((&raw const *str).cast());
    }
}
