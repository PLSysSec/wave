// use libc::{mmap, mprotect, munmap};
use libc::{PROT_NONE, PROT_READ, PROT_WRITE};
use libc::{MAP_ANONYMOUS, MAP_PRIVATE, MAP_FAILED};
use libc::c_void;
use std::ptr;
use prusti_contracts::*;
use crate::tcb::misc::bitwise_or;

// 1 << 32 = 4GB
const FOUR_GB: usize = 1 << 32;
// 1 << 33 = 8GB
const EIGHT_GB: usize = 1 << 33;

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub enum MmapProt {
//     PROT_NONE,
//     PROT_READ,
//     PROT_READWRITE,
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct Mmapping {
//     len: usize,
//     prot: MmapProt,
// }

// #[pure]
// #[trusted]
// pub fn mapping(addr: usize) -> Option<Mmapping> {
//     unimplemented!()
// }

#[trusted]
// #[requires(n <= dest.len())]
// #[ensures(forall(|i: usize|  (i < n) ==> dest[i] == c))]
pub fn memset(dest: usize, c: u8, n: usize){
    unsafe{
    libc::memset(dest as *mut c_void, c as i32, n);
    }
}



// // #[requires(addr == 0)]
// // #[requires()]
#[trusted]
//#[ensures((result != MAP_FAILED) ==> mapping(result) == Some(Mmapping(len,prot)) ]
pub fn mmap(
    addr: usize,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32, // fd to back to
    offset: i64 // offset into file to back to
) -> usize {
    unsafe{
        libc::mmap(addr as *mut c_void, len, prot, flags, fd, offset) as usize
    }
}

// #[ensures((result == 0) ==> mapping(addr) == None)]
#[trusted]
pub fn munmap(addr: usize, len: usize) -> i32 {
    unsafe{
        libc::munmap(addr as *mut libc::c_void, len)
    }
}

// TODO: bytewise permissions?
// #[ensures((result == 0) ==> )]
#[trusted]
pub fn mprotect(addr: usize, len: usize, prot: i32) -> i32 {
    unsafe{
        libc::mprotect(addr as *mut c_void, len, prot)
    }
}


// bodyless viper function
#[pure]
#[trusted]
pub fn mem_setup_correctly(memptr: usize) -> bool {
    unimplemented!()
} 

// #[pure]
// #[trusted]
// pub fn cool_ptr(memptr: usize) -> bool {
//     unimplemented!()
// } 

// #[trusted]
// #[ensures(mem_setup_correctly(result))]
fn wave_alloc_linmem() -> usize {
    let linmem_ptr = mmap(
        0,                           // let the kernel place the region anywhere
        EIGHT_GB,                    // Linmem + guard page = 8GB
        bitwise_or(PROT_READ, PROT_WRITE),      // its read/write
        bitwise_or(MAP_PRIVATE, MAP_ANONYMOUS), // should not be shared or backed-up to a file
        -1,                          // no file descrptor since we aren't backing to a file
        0,                           // this arg doesn't matter since we aren't backing to a file
    ); 
    // let x: [u8; 4] = [0,1,2,3];
    // assert!( cool_ptr((&x).as_ptr()) );
    mprotect(linmem_ptr + FOUR_GB, FOUR_GB, PROT_NONE); // Make second 4GB of linear memory a guard page
    memset(linmem_ptr, 0, FOUR_GB); // memzero
    linmem_ptr
    
}
