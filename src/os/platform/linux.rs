//! Contains call implementations that are specific to Linux/Posix
//! See src/tcb/os_specs for the raw system calls.

use crate::tcb::os_specs::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::with_ghost_var;
use prusti_contracts::*;
use syscall::syscall;



