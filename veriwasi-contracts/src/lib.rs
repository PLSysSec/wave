extern crate proc_macro;
pub use prusti_contracts::*;

#[cfg(feature = "fuzzing")]
pub use extra_args::dummy_macro as trusted;
#[cfg(feature = "fuzzing")]
pub use predicate as predicate;


macro_rules! predicate {
    ($v:tt) => {$v}
}
//pub use private::*;
