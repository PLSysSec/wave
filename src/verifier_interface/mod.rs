/// This file contains dummy implementations that do nothing when we are not verifying

#[cfg(not(feature = "verify"))]
#[macro_export]
macro_rules! effect {
    ($trace:expr, $input:expr) => {};
}

#[cfg(not(feature = "verify"))]
#[macro_export]
macro_rules! no_effect {
    ($old_trace:expr, $trace:expr) => {};
}

#[cfg(not(feature = "verify"))]
#[macro_export]
macro_rules! one_effect {
    ($old_trace:expr, $trace:expr, $( $pattern:pat )|+ $( if $guard: expr )? ) => {};
}
