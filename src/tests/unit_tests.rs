// use super::*;
// use crate::runtime::*;
// use crate::types::*;
// use crate::wrappers::*;
// use std::time::Instant;

// // some basic sanity tests
// #[cfg(test)]
// #[test]
// fn test_time_get() -> RuntimeResult<()> {
//     let mut ctx = fresh_ctx(String::from("."));
//     let ret = wasi_clock_time_get(&mut ctx, ClockId::Realtime, Timestamp::new(0))?;
//     let reference = Instant::now();

//     assert_ne!(ret, Timestamp::new(0));
//     assert_eq!(ctx.errno, RuntimeError::Success);
//     Ok(())
// }

// #[cfg(test)]
// #[test]
// fn test_res_get() -> RuntimeResult<()> {
//     let mut ctx = fresh_ctx(String::from("."));
//     let ret = wasi_clock_res_get(&mut ctx, ClockId::Realtime)?;
//     let reference = Instant::now();

//     assert_ne!(ret, Timestamp::new(0));
//     assert_eq!(ctx.errno, RuntimeError::Success);
//     Ok(())
// }
