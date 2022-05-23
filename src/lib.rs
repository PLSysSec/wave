#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_must_use,
    unused_comparisons,
    unused_parens
)]

use wave_macros::{external_call, external_method, with_ghost_var};

extern crate prusti_contracts;

mod fdmap;
#[cfg(not(feature = "verify"))] // TODO: verify this final ffi layer
pub mod lucet_frontend;
mod os;
pub mod runtime;
pub mod stats;
pub mod tcb;
#[cfg(not(feature = "verify"))]
mod tests;
pub mod types;
pub mod verifier_interface;
#[cfg(not(feature = "verify"))] // TODO: verify this final ffi layer
pub mod wasm2c_frontend;
mod writeback;
// #[with_ghost_var(trace: &mut Trace)]
mod iov;
mod path_resolution;
mod poll;
mod wrappers;
//pub mod setup_teardown;
