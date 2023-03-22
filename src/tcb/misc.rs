use crate::rvec::RVec;
// #[cfg(feature = "veify")]
// use crate::tcb::verifier::*;
use crate::types::*;
// use prusti_contracts::*;
use std::os::unix::io::AsRawFd;
use std::vec::Vec;
// use wave_macros::with_ghost_var;

///////////////////////////////// Bitwise Ops /////////////////////////////////
/// These operations are currently trusted because prusti does not handle
/// bitwise operations. However, they have no #[ensures] annotations, so they
/// cannot introduce unsoundness into our proof and so I don't expect any of
/// these functions to cause any trouble.

/// Check if The nth bit from the lsb is set (0 is lsb)
// #[trusted]
// #[pure]
// #[ensures(bv == 0 ==> result == false)]
#[flux::trusted]
#[flux::sig(fn (bv: u16, n: i32) -> bool[nth_bit_set(bv, n)])]
pub fn nth_bit_set(bv: u16, n: i32) -> bool {
    bv & (1 << n) != 0
}

// #[trusted]
// #[pure]
#[flux::trusted]
#[flux::sig(fn (bv: u32, n: u32) -> bool[nth_bit_set(bv, n)])]
pub fn nth_bit_set_u32(bv: u32, n: u32) -> bool {
    bv & (1 << n) != 0
}

/// return bv with the nth bit from the lsb set (0 is lsb)
// #[trusted]
// #[pure]
#[flux::trusted]
pub fn with_nth_bit_set(bv: u16, n: i32) -> u16 {
    bv | (1 << n)
}

// #[trusted]
// #[pure]
#[flux::trusted]
#[flux::sig(fn (bv: i32, flag: i32) -> bool{b: (b == flag_set(bv, flag)) && (b => (bv != 0)) })]
pub fn flag_set(bv: i32, flag: i32) -> bool {
    (bv & flag) == flag
}

// #[trusted]
// #[pure]
#[flux::trusted]
pub fn bitwise_and(bv1: i32, bv2: i32) -> i32 {
    bv1 & bv2
}

// #[trusted]
#[flux::trusted]
pub fn bitwise_and_i16(bv1: i16, bv2: i16) -> i16 {
    bv1 & bv2
}

// #[trusted]
#[flux::trusted]
pub fn bitwise_and_u16(bv1: u16, bv2: u16) -> u16 {
    bv1 & bv2
}

// #[trusted]
// #[pure]
#[flux::trusted]
pub fn bitwise_and_u32(bv1: u32, bv2: u32) -> u32 {
    bv1 & bv2
}

// #[trusted]
#[flux::trusted]
pub fn bitwise_and_u64(bv1: u64, bv2: u64) -> u64 {
    bv1 & bv2
}

// #[trusted]
// #[pure]
// this ensures is mostly to prove that or'ing a nonzero # returns a
// nonzero result
// #[with_ghost_var(trace: &mut Trace)]
// #[ensures(result >= bv1 && result >= bv2)]
#[flux::trusted]
pub fn bitwise_or(bv1: i32, bv2: i32) -> i32 {
    bv1 | bv2
}

// #[trusted]
#[flux::trusted]
pub fn bitwise_or_u32(bv1: u32, bv2: u32) -> u32 {
    bv1 | bv2
}

// Unsafe necessary as libc::stat is opaque. It is safe but we can replace it by implementing
// pub fn bitwise_or_u32(bv1: u32, bv2: u32) -> u32 {
// the struct ourselves if we want to avoid as much unsafe as possible.
//     bv1 | bv2
// Safety: Safe as libc::stat is valid with an all-zero byte-pattern (i.e. it is not a
// }
// reference)
// #[trusted]
#[flux::trusted]
pub fn fresh_stat() -> libc::stat {
    unsafe { std::mem::zeroed() }
}

// Unsafe necessary as libc::rusage is opaque. It is safe but we can replace it by implementing
// pub fn bitwise_or_u32(bv1: u32, bv2: u32) -> u32 {
// the struct ourselves if we want to avoid as much unsafe as possible.
//     bv1 | bv2
// Safety: Safe as libc::rusage is valid with an all-zero byte-pattern (i.e. it is not a
// }
// reference)
// #[trusted]
#[flux::trusted]
pub fn fresh_rusage() -> libc::rusage {
    unsafe { std::mem::zeroed() }
}

#[flux::trusted]
// #[requires(len >= offset)]
// #[requires(buf.len() >= start + len)]
// #[ensures(result < old(len))]
#[flux::sig(fn (buf: &RVec<u8>{buf_len: buf_len >= start + len}, start: usize, offset: usize, len: usize{len >= offset}) -> usize{result : result < len})]
pub fn first_null(buf: &RVec<u8>, start: usize, offset: usize, len: usize) -> usize {
    buf.inner[start + offset..start + len]
        .iter()
        .position(|x| *x == 0)
        .unwrap()
}

// #[trusted]
#[flux::trusted]
pub fn push_dirent_name(out_buf: &mut RVec<u8>, buf: &RVec<u8>, start: usize, len: usize) {
    out_buf
        .inner
        .extend_from_slice(&buf.inner[start..start + len])
}

// // Trusted because I need to convince prusti that clone does not alter
// // the length of vectors
// #[trusted]
// #[ensures(result.len() == old(vec.len()))]
// pub fn clone_vec_u8(vec: &Vec<u8>) -> Vec<u8> {
//     vec.clone()
// }

// TODO: should probably fail more elegantly than this
// #[trusted]
#[flux::trusted]
pub fn get_homedir_fd(s: &String) -> i32 {
    let homedir_file = std::fs::File::open(s).unwrap();
    let homedir_fd = homedir_file.as_raw_fd();

    //Need to forget file to make sure it does not get auto-closed
    //when it gets out of scope
    std::mem::forget(homedir_file);
    homedir_fd
}

// #[trusted]
#[flux::trusted]
pub fn string_to_rvec_u8(s: &String) -> RVec<u8> {
    RVec::from_vec(s.as_bytes().to_vec())
}

// #[with_ghost_var(trace: &mut Trace)]
// #[trusted]
pub fn empty_netlist() -> Netlist {
    let empty = NetEndpoint {
        protocol: WasiProto::Unknown,
        addr: 0,
        port: 0,
    };

    Netlist::new([empty, empty, empty, empty])
}

// this shouldn't need to be trusted, but prusti does not casting an enum to an int
// #[trusted]
#[flux::trusted]
pub fn as_u32(e: RuntimeError) -> u32 {
    e as u32
}

// #[trusted]
#[flux::trusted]
pub fn as_u16(e: RuntimeError) -> u16 {
    e as u16
}

// uninterpreted function
// #[trusted]
// #[pure]
// pub fn netlist_unmodified(n: &Netlist) -> bool {
//     unimplemented!();
// }

// FLUX-TODO2: see https://github.com/liquid-rust/flux/issues/285
#[flux::trusted]
pub fn unsize_arr_slice_8(arr: &[u8; 8]) -> &[u8] {
    arr
}

// FLUX-TODO2: see https://github.com/liquid-rust/flux/issues/285
#[flux::trusted]
pub fn unsize_arr_slice_4(arr: &[u8; 4]) -> &[u8] {
    arr
}
