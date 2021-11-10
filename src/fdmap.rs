use crate::tcb::misc::vec_checked_lookup;
#[cfg(feature = "verify")]
use crate::tcb::verifier::external_specs::vec::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use std::io::{stderr, stdin, stdout};
use std::os::unix::io::AsRawFd;
use RuntimeError::*;

/*
Data structure to map sandbox file descriptors to host file descriptors.
We will prove things about it's API as necessary.
*/

//TODO: should not be able to close stdin,stdout,stderr,homedir
// homedir is hardcoded to 3.

impl FdMap {
    // #[ensures (result.m.len() == MAX_SBOX_FDS)]
    #[ensures (result.reserve.len() == 0)]
    #[ensures (result.counter == 0)]
    pub fn new() -> Self {
        FdMap {
            m: vec![Err(Ebadf); MAX_SBOX_FDS as usize],
            reserve: Vec::new(),
            counter: 0,
        }
    }

    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_call(stdin)]
    // #[external_call(stdout)]
    // #[external_call(stderr)]
    // #[external_method(as_raw_fd)]
    // #[external_method(into)]
    #[requires (self.counter == 0)] //should only be called on empty fdmap
    pub fn init_std_fds(&mut self) -> RuntimeResult<()> {
        let stdin_fd = stdin().as_raw_fd(); // upcasting i32 => usize
        let stdout_fd = stdout().as_raw_fd();
        let stderr_fd = stderr().as_raw_fd();
        if (stdin_fd >= 0) && (stdout_fd >= 0) && (stderr_fd >= 0) {
            self.create((stdin_fd as usize).into());
            self.create((stdout_fd as usize).into());
            self.create((stderr_fd as usize).into());
            return Ok(());
        }
        Err(Emfile) // File descriptor failure
    }

    #[pure]
    #[requires (index < MAX_SBOX_FDS )]
    pub fn lookup(&self, index: SboxFd) -> RuntimeResult<HostFd> {
        // self.m[index as usize]
        vec_checked_lookup(&self.m, index)
    }

    #[pure]
    #[requires(index < MAX_SBOX_FDS)]
    #[ensures(result == true ==> self.lookup(index).is_ok())]
    pub fn contains(&self, index: SboxFd) -> bool {
        matches!(self.lookup(index), Ok(_))
    }

    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_call(Ok)]
    // #[external_call(Err)]
    // #[external_method(pop)]
    fn pop_fd(&mut self) -> RuntimeResult<SboxFd> {
        match self.reserve.pop() {
            Some(fd) => Ok(fd),
            None => {
                if self.counter < MAX_SBOX_FDS {
                    self.counter += 1;
                    return Ok(self.counter - 1);
                }
                Err(Emfile)
            }
        }
    }

    // #[requires(k < MAX_HOST_FDS)]
    // #[ensures (self.lookup(k) == result)]
    // #[ensures (forall(|i: usize| (i < MAX_SBOX_FDS && i != k) ==>
    //                 self.lookup(i) == old(self.lookup(i))))]
    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_call(Ok)]
    // #[requires(trace_safe(ctx, trace))]
    // #[ensures(trace_safe(ctx, trace))]
    pub fn create(&mut self, k: HostFd) -> RuntimeResult<SboxFd> {
        let s_fd = self.pop_fd()?;
        self.m[s_fd as usize] = Ok(k);
        Ok(s_fd)
    }

    #[requires(k < MAX_SBOX_FDS)]
    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_call(Err)]
    // #[external_call(init_std_fds)]
    // #[external_method(push)]
    // #[requires(trace_safe(ctx, trace))]
    // #[ensures(trace_safe(ctx, trace))]
    // #[ensures (self.lookup(k).is_err())]
    // #[ensures (forall(|i: usize| (i < MAX_SBOX_FDS && i != k) ==>
    //                 self.lookup(i) == old(self).lookup(i)))]
    pub fn delete(&mut self, k: SboxFd) {
        if let Ok(oldfd) = self.m[k as usize] {
            self.reserve.push(k);
        }
        self.m[k as usize] = Err(Ebadf);
    }

    #[requires(from < MAX_SBOX_FDS)]
    #[requires(to < MAX_SBOX_FDS)]
    pub fn shift(&mut self, from: SboxFd, to: SboxFd) {
        if let Ok(hostfd) = self.m[from as usize] {
            self.m[to as usize] = Ok(hostfd)
        }
        self.m[from as usize] = Err(Ebadf);
    }
}
