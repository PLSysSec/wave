#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_must_use,
    unused_comparisons
)]

use extra_args::{external_call, external_method, with_ghost_var};

extern crate prusti_contracts;

// If we are fuzzing the trusted code, reinterpret the #[trusted] annotation
//#[cfg(feature = "fuzz_trusted")]
//use extra_args::dummy_macro as trusted;

mod fdmap;
mod os;
mod runtime;
mod tests;
mod types;
//#[cfg(not(feature = "verify"))]
//pub mod wasm2c_frontend;
//#[cfg(feature = "verify")]
//pub mod verifier;
pub mod tcb;
pub mod verifier_interface;
// #[with_ghost_var(trace: &mut Trace)]
mod wrappers;
