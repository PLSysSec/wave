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

/*#[quickcheck_macros::quickcheck]
fn check_memcpy_to_sandbox(dst: SboxPtr, src: Vec<u8>, n: u32) -> TestResult {
    let mut ctx = fresh_ctx(".".to_string());
    let old_memlen = ctx.memlen;
    if !(src.len() == (n as usize)) {
        return TestResult::discard();
    }
    if !ctx.fits_in_lin_mem(dst, n) {
        return TestResult::discard();
    }

    let r = ctx.memcpy_to_sandbox(dst, &src, n);
    TestResult::from_bool(old_memlen == ctx.memlen)
}*/
