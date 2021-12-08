mod fuzz_trusted;
mod unit_tests;

mod test_generator;

use crate::runtime::fresh_ctx;
//use crate::tcb::verifier::*;
use crate::types::VmCtx;
use quickcheck::{Arbitrary, Gen};

#[cfg(test)]
impl Arbitrary for VmCtx {
    fn arbitrary(g: &mut Gen) -> Self {
        fresh_ctx(".".to_string())
    }
}

/*#[cfg(test)]
impl Arbitrary for Trace {
    fn arbitrary(g: &mut Gen) -> Self {
        Trace::new()
    }
}*/
