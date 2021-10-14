//#[cfg(feature = "verify")]
//use crate::verifier;

/// If we are verifying, invoke verifier, else do nothing
// #[cfg(feature = "verify")]
// use ::effect;
// #[macro_export]
// macro_rules! effect {
//     ($trace:expr, $input:expr) => {
//         verifier::effect!($trace, $input)
//     }
// }

//Dummy implementation that does nothing when we are not verifying
#[cfg(not(feature = "verify"))]
#[macro_export]
macro_rules! effect {
    ($trace:expr, $input:expr) => {};
}
