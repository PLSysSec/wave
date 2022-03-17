#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use prusti_contracts::*;
use std::os::unix::io::AsRawFd;
use std::vec::Vec;
use wave_macros::with_ghost_var;

// Trusted because I can't get the verifier to understand that
// this can't ever err and it is pretty clear it is correct.
// Can be fixed with https://viperproject.github.io/prusti-dev/user-guide/verify/pledge.html
// Used in fdmap implementation
#[trusted]
#[pure]
#[requires(index < MAX_SBOX_FDS )]
pub fn vec_checked_lookup(
    vec: &Vec<RuntimeResult<HostFd>>,
    index: SboxFd,
) -> RuntimeResult<HostFd> {
    vec[index as usize]
}

// // Trusted because I can't convince the verifier tha tthis will never panic.
// // Used in specification in src/os.rs
// #[trusted]
// #[pure]
// #[requires(r.is_ok())]
// pub fn safe_unwrap(r: &RuntimeResult<isize>) -> isize {
//     r.unwrap()
// }

///////////////////////////////// Bitwise Ops /////////////////////////////////
/// These operations are currently trusted because prusti does not handle
/// bitwise operations. However, they have no #[ensures] annotations, so they
/// cannot introduce unsoundness into our proof and so I don't expect any of  
/// these functions to cause any trouble.

/// Check if The nth bit from the lsb is set (0 is lsb)
#[trusted]
pub fn nth_bit_set(bv: u16, n: i32) -> bool {
    bv & (1 << n) != 0
}

#[trusted]
pub fn nth_bit_set_u32(bv: u32, n: u32) -> bool {
    bv & (1 << n) != 0
}

/// return bv with the nth bit from the lsb set (0 is lsb)
#[trusted]
pub fn with_nth_bit_set(bv: u16, n: i32) -> u16 {
    bv | (1 << n)
}

#[trusted]
pub fn bitwise_and(bv1: i32, bv2: i32) -> i32 {
    bv1 & bv2
}

#[trusted]
pub fn bitwise_and_i16(bv1: i16, bv2: i16) -> i16 {
    bv1 & bv2
}

#[trusted]
pub fn bitwise_and_u16(bv1: u16, bv2: u16) -> u16 {
    bv1 & bv2
}

#[trusted]
pub fn bitwise_and_u32(bv1: u32, bv2: u32) -> u32 {
    bv1 & bv2
}

#[trusted]
pub fn bitwise_and_u64(bv1: u64, bv2: u64) -> u64 {
    bv1 & bv2
}

#[trusted]
pub fn bitwise_or(bv1: i32, bv2: i32) -> i32 {
    bv1 | bv2
}

// Unsafe necessary as libc::stat is opaque. It is safe but we can replace it by implementing
// pub fn bitwise_or_u32(bv1: u32, bv2: u32) -> u32 {
// the struct ourselves if we want to avoid as much unsafe as possible.
//     bv1 | bv2
// Safety: Safe as libc::stat is valid with an all-zero byte-pattern (i.e. it is not a
// }
// reference)
#[trusted]
pub fn fresh_stat() -> libc::stat {
    unsafe { std::mem::zeroed() }
}

#[trusted]
#[requires(len >= 19)]
#[requires(buf.len() >= start + len)]
#[ensures(result < old(len))]
pub fn first_null(buf: &Vec<u8>, start: usize, len: usize) -> usize {
    buf[start + 19..start + len]
        .iter()
        .position(|x| *x == 0)
        .unwrap()
}

#[trusted]
// #[requires(buf.len() > start + len + 19)]
pub fn push_dirent_name(out_buf: &mut Vec<u8>, buf: &Vec<u8>, start: usize, len: usize) {
    out_buf.extend_from_slice(&buf[start + 19..start + 19 + len])
}

// Trusted because I need to convince prusti that clone does not alter
// the length of vectors
#[trusted]
#[ensures(result.len() == old(vec.len()))]
pub fn clone_vec_u8(vec: &Vec<u8>) -> Vec<u8> {
    vec.clone()
}

// TODO: should probably fail more elegantly than this
#[trusted]
pub fn get_homedir_fd(s: &String) -> i32 {
    let homedir_file = std::fs::File::open(s).unwrap();
    let homedir_fd = homedir_file.as_raw_fd();

    //Need to forget file to make sure it does not get auto-closed
    //when it gets out of scope
    std::mem::forget(homedir_file);
    homedir_fd
}

#[trusted]
pub fn string_to_vec_u8(s: &String) -> Vec<u8> {
    s.as_bytes().to_vec()
}

#[with_ghost_var(trace: &mut Trace)]
#[trusted]
pub fn empty_netlist() -> Netlist {
    let empty = NetEndpoint {
        protocol: WasiProto::Unknown,
        addr: 0,
        port: 0,
    };

    [empty, empty, empty, empty]
}


// this shouldn't need to be trusted, but prusti does not casting an enum to an int
#[trusted]
pub fn as_u32(e: RuntimeError) -> u32 {
    e as u32
}

#[trusted]
pub fn as_u16(e: RuntimeError) -> u16 {
    e as u16
}

// uninterpreted ghost function to attach
// #[pure]
// #[trusted]
// pub fn fd_proto(fd: HostFd) -> WasiProto {
//     unimplemented!()
// }

// #[pure]
// #[trusted]
// #[ensures(fd_proto(fd) == proto)]
// pub fn tag_proto(fd: HostFd, proto: WasiProto) -> WasiProto {
//     unimplemented!()
// }
