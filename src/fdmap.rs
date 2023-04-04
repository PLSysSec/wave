// FLUX use crate::tcb::misc::vec_checked_lookup;
// FLUX use crate::tcb::verifier::external_specs::vec::*;
// FLUX #[cfg(feature = "verify")]
// FLUX #[cfg(feature = "verify")]
// FLUX use crate::tcb::verifier::*;
// FLUX use prusti_contracts::*;
// FLUX use wave_macros::{external_calls, external_methods, with_ghost_var};

use crate::types::*;
use crate::{rvec, rvec::RVec};
use std::io::{stderr, stdin, stdout};
use std::os::unix::io::AsRawFd;
use RuntimeError::*;
// use crate::os::trace_close;
// FLUX use crate::tcb::os_specs::os_close;

/*
Data structure to map sandbox file descriptors to host file descriptors.
We will prove things about it's API as necessary.
*/

//TODO: should not be able to close stdin,stdout,stderr,homedir
// homedir is hardcoded to 3.

impl FdMap {
    #[flux::sig(fn () -> FdMap[0, 0])]
    pub fn new() -> FdMap {
        FdMap {
            m: rvec![Err(Ebadf); MAX_SBOX_FDS as usize],
            sockinfo: rvec![Err(Enotsock); MAX_SBOX_FDS as usize], // these are the host protocol domain/ty/family numbers
            reserve: RVec::new(),
            counter: 0,
        }
    }

    // #[requires (self.counter == 0)] //should only be called on empty fdmap

    #[flux::sig(fn (self: &strg { FdMap[@fm] | fm.counter == 0}) -> Result<(), RuntimeError> ensures self: FdMap)]
    pub fn init_std_fds(&mut self) -> Result<(), RuntimeError> {
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

    // #[pure]
    // #[requires (index < MAX_SBOX_FDS )]
    // pub fn lookup(&self, index: SboxFd) -> RuntimeResult<HostFd> {
    //     vec_checked_lookup(&self.m, index)
    // }

    // #[with_ghost_var(trace: &Trace)]
    // #[external_calls(vec_checked_lookup)]
    // #[ensures(result.is_ok() ==> old(v_fd) < MAX_SBOX_FDS)]
    #[flux::sig(fn (&FdMap, v_fd: SboxFd) -> Result<{HostFd | v_fd < MAX_SBOX_FDS}, RuntimeError>)]
    pub fn fd_to_native(&self, v_fd: SboxFd) -> Result<HostFd, RuntimeError> {
        if v_fd >= MAX_SBOX_FDS {
            return Err(Ebadf);
        }
        self.m[v_fd as usize]
    }

    // #[pure]
    // #[requires(index < MAX_SBOX_FDS)]
    // #[ensures(result == true ==> self.lookup(index).is_ok())]
    // pub fn contains(&self, index: SboxFd) -> bool {
    //     matches!(self.lookup(index), Ok(_))
    // }

    #[flux::sig(fn (self: &strg FdMap[@fd]) -> Result<SboxFdSafe, RuntimeError> ensures self: FdMap)]
    fn pop_fd(&mut self) -> Result<SboxFdSafe, RuntimeError> {
        if self.reserve.len() > 0 {
            Ok(self.reserve.pop())
        } else {
            if self.counter < MAX_SBOX_FDS {
                self.counter += 1;
                return Ok(self.counter - 1);
            }
            Err(Emfile)
        }
    }

    #[flux::sig(fn (self: &strg FdMap[@dummy], k: HostFd) -> Result<SboxFd, RuntimeError> ensures self: FdMap)]
    pub fn create(&mut self, k: HostFd) -> Result<SboxFd, RuntimeError> {
        let s_fd = self.pop_fd()?;
        self.m[s_fd as usize] = Ok(k);
        Ok(s_fd)
    }

    #[flux::sig(fn (self: &strg FdMap[@dummy], k: HostFd, proto: WasiProto) -> Result<SboxFd, RuntimeError> ensures self: FdMap)]
    pub fn create_sock(&mut self, k: HostFd, proto: WasiProto) -> Result<SboxFd, RuntimeError> {
        let s_fd = self.pop_fd()?;
        self.m[s_fd as usize] = Ok(k);
        self.sockinfo[s_fd as usize] = Ok(proto);
        Ok(s_fd)
    }

    // #[requires(k < MAX_SBOX_FDS)]
    // FLUX-TODO2 open-mut-ref
    #[flux::sig(fn (self: &strg FdMap, k: SboxFdSafe) ensures self: FdMap)]
    pub fn delete(&mut self, k: SboxFdSafe) {
        if let Ok(oldfd) = self.m[k as usize] {
            self.reserve.push(k);
        }
        self.m[k as usize] = Err(Ebadf);
    }

    // #[requires(from < MAX_SBOX_FDS)]
    // #[requires(to < MAX_SBOX_FDS)]
    #[flux::sig(fn (self: &mut FdMap[@dummy], from: SboxFdSafe, to: SboxFdSafe))]
    pub fn shift(&mut self, from: SboxFdSafe, to: SboxFdSafe) {
        if let Ok(hostfd) = self.m[from as usize] {
            self.m[to as usize] = Ok(hostfd)
        }
        self.m[from as usize] = Err(Ebadf);
    }
}
