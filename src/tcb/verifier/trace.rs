use prusti_contracts::*;

/// Goal:
/// Testing the possibility of using an effects system and verifying effects
/// via an invariant on the trace

/// Findings:
/// 1. Cannot compare refs to enums or structs inside predicate (unless you use old)
/// 2. Trace should not be part of context (no unnecessary mutability)
/// 3. Trace should be append only (as this one is)
/// Cannot add an additional trace argument conditional on compiliation,
/// 2 possible solutions:
///     1. make trace exist at runtime, just don't interact with it: ==> we have to do this
///     2. make trace global: ==> Rust does not allow this

// TODO: combine into a single variadic macro

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
macro_rules! do_effect {
    ($trace:expr, $input:expr) => {
        if cfg!(feature = "verify") {
            $trace.push($input);
        }
    };
}

/*#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Effect {
    ReadN(usize),  // read into `addr` `count` bytes
    WriteN(usize), // write into `addr` `count` bytes
    Shutdown,
    FdAccess, // TODO: should this store the HostFd?
    PathAccess,
}*/

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum EffectType {
    ReadN,
    WriteN,
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
        }
    };
    ($typ:ident, $f1:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: 0,
            f3: 0,
        }
    };
    ($typ:ident, $f1:pat, $f2:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: $f2,
            f3: 0,
        }
    };
    ($typ:ident, $f1:pat, $f2:pat, $f3:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: $f2,
            f3: $f3,
        }
    };
}

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
}

/*SafePtr --> newtype around pointer
Track length and is safe
    - basically it came from a safe struct so we know it is safe?
    - statically tracked size*/

/*
 * 1. Invariants:
 *    - make sure fds are actually fds that are distinct from sandbox filedescriptors
 *    - passing paths to the os: make sure paths are within the home directory of the
 *          sandbox
 *          - try not trusted
 *    - multi-threading? maybe
 *          -
 *    - try prove functional correctness for one call
 *      - encode posix spec.
 *      - can try
 *      - encoding sandbox memory isolation on top of that
 *      - plus any added invariants
 *      - path_open might be more complicated
 */
