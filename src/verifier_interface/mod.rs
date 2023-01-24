/// This file contains dummy implementations that do nothing when we are not verifying

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! effect {
    ($trace:expr, $input:expr) => {};
}

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! path_effect {
    ($trace:expr, $input:expr) => {};
}

// #[cfg(not(feature = "verify"))]
// #[macro_export]
// macro_rules! do_effect {
//     ($trace:expr, $input:expr) => {};
// }

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! effects {
    ($trace:expr, $input:expr) => {};
}

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! map_effects {
    ($trace:expr, $input:expr) => {};
}

// Dummy timing functions that should not exist during verification
// #[cfg(feature = "verify")]
#[inline]
pub fn start_timer() -> u64 {
    0
}

// #[cfg(feature = "verify")]
#[inline]
pub fn stop_timer() -> u64 {
    0
}

// #[cfg(feature = "verify")]
#[inline]
pub fn push_hostcall_result(_name: &str, _start: u64, _end: u64) {}

// #[cfg(feature = "verify")]
#[inline]
pub fn push_syscall_result(_name: &str, _start: u64, _end: u64) {}

// #[cfg(feature = "verify")]
pub fn output_hostcall_perf_results() {}

// #[cfg(feature = "verify")]
pub fn output_syscall_perf_results() {}
