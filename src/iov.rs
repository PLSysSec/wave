use prusti_contracts::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::{effect, effects};
use crate::types::*;

// #[macro_export]
// macro_rules! effects_from_iov {
//     ($trace:expr, $input:expr) => {
//         if cfg!(feature = "verify") {
//             $trace.push($input);
//         }
//     };
// }

#[cfg(feature = "verify")]
predicate! {
    pub fn iov_eq(ev: Effect, iov: &NativeIoVec) -> bool {
        match ev {
            effect!(ReadN,addr,count) => 
                addr == iov.iov_base && 
                count == iov.iov_len,
            _ => false,
        }
    }
}

