

#[cfg_attr(target_os = "linux",
           path="platform/linux.rs")]
#[cfg_attr(target_os = "macos",
           path="platform/macos.rs")]
mod platform;
pub use platform::*;
