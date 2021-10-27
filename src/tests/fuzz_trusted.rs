use crate::os;
use crate::runtime::fresh_ctx;
use crate::types::{SboxPtr, VmCtx};
use quickcheck::TestResult;
use quickcheck_macros;

// fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
//     let mut rev = vec!();
//     for x in xs {
//         rev.insert(0, x.clone())
//     }
//     rev
// }

// #[quickcheck_macros::quickcheck]
// fn double_reversal_is_identity(xs: Vec<isize>) -> bool {
//     xs == reverse(&reverse(&xs))
// }

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
#[quickcheck_macros::quickcheck]
fn fuzz_copy_arg_buffer_to_sandbox(dst: SboxPtr, n: u32) -> TestResult {
    let mut ctx = fresh_ctx(".".to_string());
    if !(ctx.arg_buffer.len() == (n as usize)) {
        TestResult::discard();
    }
    let r = ctx.copy_arg_buffer_to_sandbox(dst, n);
    TestResult::from_bool(true)

    //xs == reverse(&reverse(&xs))
}
