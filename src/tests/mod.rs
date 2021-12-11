mod fuzz_trusted;
mod unit_tests;

mod test_generator;
mod test_gen_trace;

use crate::runtime::fresh_ctx;
use crate::types::Stat;
//use crate::tcb::verifier::*;
use crate::types::VmCtx;
use quickcheck::{Arbitrary, Gen};

/// Any common initialization for the tests (e.g. changing the working directory)
pub fn init() {
    // this will only actually happen once, but just call it everytime
    // it will fail but that is fine...
    std::env::set_current_dir("./fuzz-dir");
}

#[cfg(test)]
impl Arbitrary for VmCtx {
    fn arbitrary(g: &mut Gen) -> Self {
        fresh_ctx(".".to_string())
    }
}

impl Arbitrary for Stat {
    fn arbitrary(g: &mut Gen) -> Self {
        panic!("TODO");
    }
}

/*#[cfg(test)]
impl Arbitrary for Trace {
    fn arbitrary(g: &mut Gen) -> Self {
        Trace::new()
    }
}*/
