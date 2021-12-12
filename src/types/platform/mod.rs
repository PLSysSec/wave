
#[cfg_attr(all(target_os = "macos",
               target_arch = "aarch64"),
           path="macos-aarch64.rs")]
#[cfg_attr(all(target_os = "linux",
               target_arch = "x86_64"),
           path="linux-x86_64.rs")]
mod platform_impl;

pub use platform_impl::*;
