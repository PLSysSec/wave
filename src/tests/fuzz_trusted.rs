use crate::os;
use crate::runtime::fresh_ctx;
use crate::types::{SboxPtr, VmCtx};
use quickcheck::{QuickCheck, TestResult};
use quickcheck_macros;

// impl Arbitrary for VmCtx {
//     fn arbitrary(g: &mut Gen) -> Point {
//         VmCtx {
//             mem: Vec<u8>::arbitrary(g),
//             memlen,
//             fdmap,
//             homedir,
//             errno: Success,
//             arg_buffer,
//             argc,
//             env_buffer,
//             envc,
//         }
//         // Point {
//         //     x: i32::arbitrary(g),
//         //     y: i32::arbitrary(g),
//         // }
//     }
// }

// This test has no ensures, so I guess I don't need to check anything?
// guess we're just testing that it doesn't segfault
// #[quickcheck_macros::quickcheck]
// fn check_copy_arg_buffer_to_sandbox(dst: SboxPtr, n: u32) -> TestResult {
//     let mut ctx = fresh_ctx(".".to_string());
//     if !(ctx.arg_buffer.len() == (n as usize)) {
//         TestResult::discard();
//     }
//     let r = ctx.copy_arg_buffer_to_sandbox(dst, n);
//     TestResult::from_bool(true)
// }

#[quickcheck_macros::quickcheck]
fn check_memcpy_to_sandbox(dst: SboxPtr, src: Vec<u8>, n: u32) -> TestResult {
    let mut ctx = fresh_ctx(".".to_string());
    let old_memlen = ctx.memlen;
    if !(src.len() == (n as usize)) {
        TestResult::discard();
    }
    if (!ctx.fits_in_lin_mem(dst, n)) {
        TestResult::discard();
    }
    let r = ctx.copy_arg_buffer_to_sandbox(dst, n);
    TestResult::from_bool(old_memlen == ctx.memlen)
}

/*
    #[with_ghost_var(trace: &mut Trace)]
    #[requires(trace_safe(self, trace))]
    #[ensures(trace_safe(self, trace))]
    #[ensures(old(self.memlen) == self.memlen)]
    // #[requires(dst < (self.memlen as u32) )]
    // #[requires(dst + n < (self.memlen as u32) )]
    pub fn memcpy_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) {
        unsafe {
            copy(
                src.as_ptr(),
                self.mem.as_mut_ptr().offset(dst as isize),
                n as usize,
            )
        };
    }
*/
