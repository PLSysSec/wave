use crate::types::*;
use prusti_contracts::*;
use std::vec::Vec;

// Trusted because I can't get the verifier to understand that
// this can't ever err and it is pretty clear it is correct.
// Can be fixed with https://viperproject.github.io/prusti-dev/user-guide/verify/pledge.html
// Used in fdmap implementation
#[trusted]
#[pure]
#[requires (index < MAX_SBOX_FDS )]
pub fn vec_checked_lookup(
    vec: &Vec<RuntimeResult<HostFd>>,
    index: SboxFd,
) -> RuntimeResult<HostFd> {
    vec[index as usize]
}

/// The nth bit from the lsb is set (0 is lsb)
// #[trusted]
// pub fn u8_nth_bit_set(bv: u8, n: i32) -> bool {
//     nth_bit_set(bv.into(), n)
// }

/// The nth bit from the lsb is set
#[trusted]
pub fn nth_bit_set(bv: u16, n: i32) -> bool {
    bv & (1 << n) != 0
}
