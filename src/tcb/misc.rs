use crate::types::*;
use prusti_contracts::*;
use std::vec::Vec;

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
pub fn bitwise_and_u32(bv1: u32, bv2: u32) -> u32 {
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
