#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, effects};
use prusti_contracts::*;

// #[macro_export]
// macro_rules! effects_from_iovs {
//     ($trace:expr, $buf:expr) => {
//         $trace.len() == old($trace.len() + $buf.len()) &&
//         forall(|i: usize| (i >= $trace.len()) ||
//         {
//             if i < old($trace.len())
//                 { $trace.lookup(i) == old($trace.lookup(i)) }
//             else
//             {
//                 let this = $buf.lookup(i - old($trace.len()));
//                 let ev = $trace.lookup(i);
//                 iov_eq(ev, &this)
//             }
//         }
//     )
//     }
// }

// #[cfg(feature = "verify")]
// predicate! {
//     pub fn effects_from_iovs(old_trace: &Trace, trace: &Trace, buf: &NativeIoVecs) -> bool {
//         trace.len() == old_trace.len() + buf.len() &&
//         forall(|i: usize| (i < trace.len()) ==>
//         {
//             if i < old(trace.len())
//                 { trace.lookup(i) == old_trace.lookup(i) }
//             else
//             {
//                 let this = buf.lookup(i - old_trace.len());
//                 let ev = trace.lookup(i);
//                 iov_eq(ev, &this)
//             }
//         }
//         )
//     }
// }

#[cfg(feature = "verify")]
predicate! {
    pub fn iov_eq_read(ev: Effect, iov: &NativeIoVec) -> bool {
        match ev {
            effect!(ReadN,addr,count) =>
                addr == iov.iov_base &&
                count == iov.iov_len,
            _ => false,
        }
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn iov_eq_write(ev: Effect, iov: &NativeIoVec) -> bool {
        match ev {
            effect!(WriteN,addr,count) =>
                addr == iov.iov_base &&
                count == iov.iov_len,
            _ => false,
        }
    }
}
