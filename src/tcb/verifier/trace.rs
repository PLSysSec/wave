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

/*#[macro_export]
macro_rules! effect {
    ($trace:expr, $input:expr) => {
        if cfg!(feature = "verify") {
            $trace.push($input);
        }
    };
}*/

//TODO: wrap into a single variadic macro / predicate?
#[cfg(feature = "verify")]
predicate! {
    pub fn takes_no_steps(old_trace: &Trace, trace: &Trace) -> bool {
        // The trace is the same length
        trace.len() == old_trace.len() &&
        // And hasn't been changed
        forall(|i: usize| (i < old_trace.len()) ==>
            trace.lookup(i) == old_trace.lookup(i))
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn takes_one_step(old_trace: &Trace, trace: &Trace) -> bool {
        // We added 1 more step
        trace.len() == old_trace.len() + 1 &&
        // But the other effects were not affected
        forall(|i: usize| (i < old_trace.len()) ==>
            trace.lookup(i) == old_trace.lookup(i))
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn takes_two_steps(old_trace: &Trace, trace: &Trace) -> bool {
        // We added 2 more steps
        trace.len() == old_trace.len() + 2 &&
        // But the other effects were not affected
        forall(|i: usize| (i < old_trace.len()) ==>
            trace.lookup(i) == old_trace.lookup(i))
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn takes_three_steps(old_trace: &Trace, trace: &Trace) -> bool {
        // We added 2 more steps
        trace.len() == old_trace.len() + 3 &&
        // But the other effects were not affected
        forall(|i: usize| (i < old_trace.len()) ==>
            trace.lookup(i) == old_trace.lookup(i))
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn takes_four_steps(old_trace: &Trace, trace: &Trace) -> bool {
        // We added 2 more steps
        trace.len() == old_trace.len() + 4 &&
        // But the other effects were not affected
        forall(|i: usize| (i < old_trace.len()) ==>
            trace.lookup(i) == old_trace.lookup(i))
    }
}

/// Enforce that no effect occured
#[macro_export]
macro_rules! no_effect {
    ($old_trace:expr, $trace:expr) => {
        takes_no_steps($old_trace, $trace)
    };
}

// TODO: combine into a single variadic macro

/// Enforce that 1 effect occured, and that effect matches "pattern" and possible "guard"
#[macro_export]
macro_rules! one_effect {
    ($old_trace:expr, $trace:expr, $( $pattern: pat_param )|+ $( if $guard: expr )? ) => {
        takes_one_step($old_trace, $trace) //&&

        && match $trace.lookup($trace.len() - 1) {
            $( $pattern )|+ => $($guard &&)? true,
            _ => false,
        }
    };
}

/// Enforce that 2 effects occured, and that they match "pattern1" and "pattern2"
#[macro_export]
macro_rules! two_effects {
    ($old_trace:expr, $trace:expr, $( $pattern1: pat_param )|+ $( if $guard1: expr )?, $( $pattern2: pat_param )|+ $( if $guard2: expr )?) => {
        takes_two_steps($old_trace, $trace)
        && match $trace.lookup($trace.len() - 2) {
            $( $pattern1 )|+ => $($guard1 &&)? true,
            _ => false,
        }
        && match $trace.lookup($trace.len() - 1) {
            $( $pattern2 )|+ => $($guard2 &&)? true,
            _ => false,
        }
            // && matches!($trace.lookup($trace.len() - 2), $pattern1)
            // && matches!($trace.lookup($trace.len() - 1), $pattern2)
    };
}

/// Enforce that 3 effects occured, and that they match the patterns specified
#[macro_export]
macro_rules! three_effects {
    ($old_trace:expr, $trace:expr, $( $pattern1: pat_param )|+ $( if $guard1: expr )?, $( $pattern2: pat_param )|+ $( if $guard2: expr )?, $( $pattern3: pat_param )|+ $( if $guard3: expr )?) => {
        takes_three_steps($old_trace, $trace)
        && match $trace.lookup($trace.len() - 3) {
            $( $pattern1 )|+ => $($guard1 &&)? true,
            _ => false,
        }
        && match $trace.lookup($trace.len() - 2) {
            $( $pattern2 )|+ => $($guard2 &&)? true,
            _ => false,
        }
        && match $trace.lookup($trace.len() - 1) {
            $( $pattern3 )|+ => $($guard3 &&)? true,
            _ => false,
        }
    };
}

#[macro_export]
macro_rules! four_effects {
    ($old_trace:expr, $trace:expr,  $( $pattern1: pat_param )|+ $( if $guard1: expr )?, $( $pattern2: pat_param )|+ $( if $guard2: expr )?, $( $pattern3: pat_param )|+ $( if $guard3: expr )?,  $( $pattern4: pat_param )|+ $( if $guard4: expr )?) => {
        takes_four_steps($old_trace, $trace)
        && match $trace.lookup($trace.len() - 4) {
            $( $pattern1 )|+ => $($guard1 &&)? true,
            _ => false,
        }
        && match $trace.lookup($trace.len() - 3) {
            $( $pattern2 )|+ => $($guard2 &&)? true,
            _ => false,
        }
        && match $trace.lookup($trace.len() - 2) {
            $( $pattern3 )|+ => $($guard3 &&)? true,
            _ => false,
        }
        && match $trace.lookup($trace.len() - 1) {
            $( $pattern4 )|+ => $($guard4 &&)? true,
            _ => false,
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
    PathAccess,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Effect {
    pub typ: EffectType,
    pub f1: usize,
    pub f2: usize,
}

// TODO: I think this has to become a proc macro if we don't wanna expand every case manually...
#[macro_export]
macro_rules! effect {
    ($typ:ident) => {
        Effect {
            typ: EffectType::$typ,
            f1: 0,
            f2: 0,
        }
    };
    ($typ:ident, $f1:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: 0,
        }
    };
    ($typ:ident, $f1:pat, $f2:pat) => {
        Effect {
            typ: EffectType::$typ,
            f1: $f1,
            f2: $f2,
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
