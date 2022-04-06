// TODO(emlaufer): Are there differences between x86_64 and aarch64 to handle?
#[cfg_attr(target_os = "macos", path = "macos-aarch64.rs")]
#[cfg_attr(
    all(target_os = "linux", target_arch = "x86_64"),
    path = "linux-x86_64.rs"
)]
mod platform_impl;

pub use platform_impl::*;
