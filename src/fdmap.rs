use crate::tcb::misc::vec_checked_lookup;
#[cfg(feature = "verify")]
use crate::tcb::verifier::external_specs::vec::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use prusti_contracts::*;
use std::io::{stderr, stdin, stdout};
use std::os::unix::io::AsRawFd;
use wave_macros::{external_calls, external_methods, with_ghost_var};
use RuntimeError::*;
// use crate::os::trace_close;
use crate::tcb::os_specs::os_close;

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
            sockinfo: vec![Err(Enotsock); MAX_SBOX_FDS as usize], // these are the host protocol domain/ty/family numbers
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
        let stdin_fd = stdin().as_raw_fd();
        let stdout_fd = stdout().as_raw_fd();
        let stderr_fd = stderr().as_raw_fd();
        if (stdin_fd >= 0) && (stdout_fd >= 0) && (stderr_fd >= 0) {
            // upcasting i32 => usize is safe since we checked that it is positive
            // viper overflow checker would yell at us if this was not the case
            self.create(HostFd::from_raw(stdin_fd as usize));
            self.create(HostFd::from_raw(stdout_fd as usize));
            self.create(HostFd::from_raw(stderr_fd as usize));
            return Ok(());
        }
        Err(Emfile) // File descriptor failure
    }

    #[pure]
    #[requires (index < MAX_SBOX_FDS )]
    pub fn lookup(&self, index: SboxFd) -> RuntimeResult<HostFd> {
        vec_checked_lookup(&self.m, index)
    }

    #[with_ghost_var(trace: &Trace)]
    #[external_calls(vec_checked_lookup)]
    // #[pure]
    #[ensures(result.is_ok() ==> old(v_fd) < MAX_SBOX_FDS)]
    pub fn fd_to_native(&self, v_fd: SboxFd) -> RuntimeResult<HostFd> {
        if v_fd >= MAX_SBOX_FDS {
            return Err(Ebadf);
        }
        // self.m[idx as usize]
        vec_checked_lookup(&self.m, v_fd)
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

    pub fn create_sock(&mut self, k: HostFd, proto: WasiProto) -> RuntimeResult<SboxFd> {
        let s_fd = self.pop_fd()?;
        self.m[s_fd as usize] = Ok(k);
        self.sockinfo[s_fd as usize] = Ok(proto);
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

    // // auto drop open file descriptors and shutdown sockets
    // #[with_ghost_var(trace: &mut Trace)]
    // // #[requires(ctx_safe(self))]
    // // #[requires(trace_safe(trace, self))]
    // // #[ensures(ctx_safe(self))]
    // // #[ensures(trace_safe(trace, self))]
    // #[external_methods(lookup)]
    // fn drop(&mut self) {
    //     let mut idx = 3; // not stdin,stdout,stderr 
    //     while idx < MAX_SBOX_FDS {
    //         // body_invariant!(ctx_safe(self));
    //         // body_invariant!(trace_safe(trace, self));
    //         match self.lookup(idx) {
    //             Ok(fd) => { os_close(fd.to_raw()); }
    //             _ => (),
    //         }
    //         idx += 1;
    //     }
    // }
    
}
