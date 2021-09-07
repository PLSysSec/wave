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

macro_rules! effect {
    ($trace:expr, $input:expr) => {
        if cfg!(feature = "verify") {
            $trace.push($input);
        }
    };
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Effect {
    ReadN { count: usize },  // read into `addr` `count` bytes
    WriteN { count: usize }, // write into `addr` `count` bytes
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

struct ServerCtx {
    pub read_addr: u32,  // ip to read from
    pub write_addr: u32, // ip to write from
}
