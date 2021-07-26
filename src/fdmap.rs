#[cfg(feature = "verify")]
use crate::external_specs::vec::*;
use crate::types::*;
use prusti_contracts::*;
use RuntimeError::*;

/*
Data structure to map sandbox file descriptors to host file descriptors.
We will prove things about it's API as necessary.
*/

impl FdMap {
    // #[trusted]
    // #[ensures (result.m.len() == MAX_SBOX_FDS)]
    #[ensures (result.reserve.len() == 0)]
    pub fn new() -> Self {
        FdMap {
            m: vec![Err(Ebadf); MAX_SBOX_FDS],
            reserve: Vec::new(),
            counter: 0,
        }
    }

    // Trusted because I can't get the verifier to understand that
    // this can't ever err and it is pretty clear it is correct.
    // Can be fixed with https://viperproject.github.io/prusti-dev/user-guide/verify/pledge.html
    #[trusted]
    #[pure]
    #[requires (index < MAX_SBOX_FDS )]
    pub fn lookup(&self, index: SboxFd) -> RuntimeResult<HostFd> {
        self.m[index]
    }

    // #[trusted]
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

    // #[trusted]
    // #[requires(k < MAX_HOST_FDS)]
    // #[ensures (self.lookup(k) == result)]
    // #[ensures (forall(|i: usize| (i < MAX_SBOX_FDS && i != k) ==>
    //                 self.lookup(i) == old(self.lookup(i))))]
    pub fn create(&mut self, k: HostFd) -> RuntimeResult<SboxFd> {
        let s_fd = self.pop_fd()?;
        self.m[s_fd] = Ok(k);
        Ok(s_fd)
    }

    // #[trusted]
    #[requires(k < MAX_SBOX_FDS)]
    // #[ensures (self.lookup(k).is_err())]
    // #[ensures (forall(|i: usize| (i < MAX_SBOX_FDS && i != k) ==>
    //                 self.lookup(i) == old(self).lookup(i)))]
    pub fn delete(&mut self, k: SboxFd) {
        if let Ok(oldfd) = self.m[k] {
            self.reserve.push(k);
        }
        self.m[k] = Err(Ebadf);
    }
}
