use prusti_contracts::*;

#[cfg(feature = "verify")]
predicate! {
    pub fn takes_n_steps(old_trace: &Trace, trace: &Trace, n: usize) -> bool {
        n >= 0 &&
        // We added n more steps
        trace.len() == old_trace.len() + n &&
        // But the other effects were not affected
        forall(|i: usize| (i < old_trace.len()) ==>
            trace.lookup(i) == old_trace.lookup(i))
    }
}

// From https://danielkeep.github.io/tlborm/book/blk-counting.html
// Simple counting macro for number of trace effects. We only have a couple
// effects per computation so we don't need anything complicated.
#[macro_export]
macro_rules! count_guards {
    () => {0usize};
    ($($pattern: pat_param)|+ $( if $guard: expr)?) => {1usize};
    ($($pattern: pat_param)|+ $( if $guard: expr)?, $($tail:tt)*) => {1usize + $crate::count_guards!($($tail)*)};
}

#[macro_export]
macro_rules! effects {
    ($old_trace:expr, $trace:expr) => {
        takes_n_steps($old_trace, $trace, 0)
    };
    ($old_trace:expr, $trace:expr, $($tail:tt)*) => {
        takes_n_steps($old_trace, $trace, $crate::count_guards!($($tail)*)) &&
            effects!(@munch $old_trace, $trace, $($tail)*)
    };
    (@munch $old_trace:expr, $trace:expr) => {
        true
    };
    (@munch $old_trace:expr, $trace:expr, $($pattern: pat_param)|+ $( if $guard: expr)?) => {
        match $trace.lookup($trace.len() - 1) {
            $( $pattern )|+ => $($guard &&)? true,
            _ => false,
        }
    };
    (@munch $old_trace:expr, $trace:expr, $($pattern: pat_param)|+ $( if $guard: expr)?, $($tail:tt)*) => {
        match $trace.lookup($trace.len() - (1 + $crate::count_guards!($($tail)*))) {
            $( $pattern )|+ => $($guard &&)? true,
            _ => false,
        } && effects!(@munch $old_trace, $trace, $($tail)*)
    };
}

#[macro_export]
macro_rules! map_effects {
    ($old_trace:expr, $trace:expr, $struct_iter:expr, $cnt:expr, $($pattern: pat_param)|+ $( if $guard: expr)?) => {
        takes_n_steps($old_trace, $trace, $cnt) &&
        forall(|idx: usize|  (idx < $cnt) ==>
            let this = $struct_iter.lookup(idx);
            match $trace.lookup($old_trace.len() + idx) {
                $( $pattern )|+ => $($guard &&)? true,
                _ => false,
            }
        )
    };
}

#[macro_export]
macro_rules! do_effect {
    ($trace:expr, $input:expr) => {
        if cfg!(feature = "verify") {
            $trace.push($input);
        }
    };
}

#[cfg(feature = "test")]
use crate::predicate;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum EffectType {
    ReadMem,
    WriteMem,
    Shutdown,
    FdAccess,
    PathAccessAt,
    NetAccess,
    SockCreation,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Effect {
    pub typ: EffectType,
    pub f1: usize,
    pub f2: usize,
    pub f3: usize,
    pub p: Option<[u8; 4096]>,
    pub should_follow: Option<bool>,
}

// TODO: I think this has to become a proc macro if we don't wanna expand every case manually...
#[macro_export]
macro_rules! effect {
    ($typ:ident) => {
        Effect {
            typ: EffectType::$typ,
            f1: 0,
            f2: 0,
            f3: 0,
            p: None,
            should_follow: None,
        }
    };
    ($typ:ident, $f1:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: 0,
            f3: 0,
            p: None,
            should_follow: None,
        }
    };
    ($typ:ident, $f1:pat, $f2:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: $f2,
            f3: 0,
            p: None,
            should_follow: None,
        }
    };
    ($typ:ident, $f1:pat, $f2:pat, $f3:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: $f2,
            f3: $f3,
            p: None,
            should_follow: None,
        }
    };
}

// macro for passing in paths
#[macro_export]
macro_rules! path_effect {
    ($typ:ident, $f1:pat, $f2:pat, $f3:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: 0,
            f3: 0,
            p: Some($f2),
            should_follow: Some($f3),
        }
    };
}

// #[trusted]
// #[pure]
// #[requires(index < MAX_SBOX_FDS )]
// pub fn vec_u8_lookup(
//     vec: &Vec<u8>,
//     index: usize,
// ) -> RuntimeResult<HostFd> {
//     vec[index as usize]
// }

// use crate::tcb::misc::vec_checked_lookup;
// #[cfg(feature = "verify")]
// predicate! {
//     pub fn vec_is_eq(v0: &Vec<u8>, v1: &Vec<u8>) -> bool {
//         v0.len() == v1.len() &&
//         forall(|i: usize|
//             (i < v0.len() ==> (
//                 vec_u8_lookup(v0, i) == vec_u8_lookup(v1, i)
//             ))
//         )
//     }
// }

pub struct Trace {
    v: Vec<Effect>,
}

impl Trace {
    // Encoded as body-less Viper function
    #[trusted]
    #[pure]
    pub fn len(&self) -> usize {
        self.v.len()
    }

    // Encoded as body-less Viper method
    #[trusted]
    #[ensures(result.len() == 0)]
    pub fn new() -> Self {
        Trace { v: Vec::new() }
    }

    // Encoded as body-less Viper function
    #[trusted]
    #[pure]
    #[requires(index < self.len())]
    pub fn lookup(&self, index: usize) -> Effect {
        self.v[index]
    }

    #[trusted]
    #[ensures(self.len() == old(self.len()) + 1)]
    #[ensures(self.lookup(old(self.len())) == old(value))]
    #[ensures(forall(|i: usize| (i < old(self.len())) ==>
                    self.lookup(i) == old(self.lookup(i))))]
    pub fn push(&mut self, value: Effect) {
        self.v.push(value);
    }

    // #[trusted]
    // #[ensures(self.num_paths() == old(self.num_paths()) + 1)]
    // #[ensures(self.lookup_path(old(self.num_paths())) == old(&value))]
    // #[ensures(forall(|i: usize| (i < old(self.num_paths())) ==>
    //                 self.lookup_path(i) == old(self.lookup_path(i))))]
    // pub fn push_path(&mut self, value: Vec<u8>) {
    //     self.paths.push(value);
    // }
}
