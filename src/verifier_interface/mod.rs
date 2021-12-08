/// This file contains dummy implementations that do nothing when we are not verifying

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! effect {
    ($trace:expr, $input:expr) => {};
}

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! no_effect {
    ($old_trace:expr, $trace:expr) => {};
}

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! one_effect {
    ($old_trace:expr, $trace:expr, $( $pattern:pat )|+ $( if $guard: expr )? ) => {};
}

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! two_effects {
    ($old_trace:expr, $trace:expr, $( $pattern:pat )|+ $( if $guard: expr )? ) => {};
}

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! three_effects {
    ($old_trace:expr, $trace:expr, $( $pattern:pat )|+ $( if $guard: expr )? ) => {};
}

#[cfg(not(any(feature = "verify", test)))]
#[macro_export]
macro_rules! four_effects {
    ($old_trace:expr, $trace:expr, $( $pattern:pat )|+ $( if $guard: expr )? ) => {};
}
