#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_must_use,
    unused_comparisons
)]

use extra_args::{external_call, external_method, with_ghost_var};

extern crate prusti_contracts;

// If we are fuzzing the trusted code, reinterpret the trusted annotation
//#[cfg(feature = "fuzz_trusted")]
//use extra_args::dummy_macro as trusted;

mod fdmap;
mod os;
mod runtime;
//#[cfg(not(feature = "verify"))]
pub mod stats;
pub mod tcb;
mod tests;
mod types;
pub mod verifier_interface;
#[cfg(not(feature = "verify"))]
pub mod wasm2c_frontend;
mod writeback;
// #[with_ghost_var(trace: &mut Trace)]
mod wrappers;
