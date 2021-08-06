#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_must_use,
    unused_comparisons
)]

extern crate prusti_contracts;

// external specs mod will crash if we are compiling without the verifier
// so only include when verifying, not when running
#[cfg(feature = "verify")]
mod external_specs;
mod fdmap;
mod os;
mod runtime;
mod tests;
mod types;
#[cfg(not(feature = "verify"))]
pub mod wasm2c_frontend;
mod wrappers;
