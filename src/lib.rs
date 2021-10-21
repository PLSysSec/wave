#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_must_use,
    unused_comparisons
)]

use extra_args::{external_call, external_method, with_ghost_var};

extern crate prusti_contracts;

// external specs mod will crash if we are compiling without the verifier
// so only include when verifying, not when running
//#[cfg(feature = "verify")]
//mod external_specs;
mod fdmap;
mod os;
mod runtime;
//mod spec;
mod tests;
//mod trace;
mod types;
//#[cfg(not(feature = "verify"))]
//pub mod wasm2c_frontend;
#[cfg(feature = "verify")]
pub mod verifier;
pub mod verifier_interface;
// #[with_ghost_var(trace: &mut Trace)]
mod wrappers;
