use crate::external_specs::vec::*;
use crate::types::*;
use prusti_contracts::*;
use RuntimeError::*;

/*
new
translate
fresh_fd
create
remove
*/

impl FdMap {
    #[trusted]
    #[ensures (result.m.len() == MAX_SBOX_FDS)]
    #[ensures (result.reserve.len() == 0)]
    pub fn new() -> Self {
        FdMap {
            m: vec![Err(Ebadf); MAX_SBOX_FDS],
            reserve: Vec::new(),
            counter: 0,
        }
    }

    #[trusted]
    #[pure]
    #[requires (index < MAX_SBOX_FDS )]
    pub fn lookup(&self, index: SboxFd) -> RuntimeResult<HostFd> {
        self.m[index]
    }

    #[trusted]
    fn pop_fd(&mut self) -> RuntimeResult<HostFd> {
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

    #[trusted]
    #[requires(k < MAX_SBOX_FDS)]
    #[ensures (self.lookup(k) == result)]
    #[ensures (forall(|i: usize| (i < MAX_SBOX_FDS && i != k) ==>
                    self.lookup(i) == old(self.lookup(i))))]
    pub fn create(&mut self, k: SboxFd) -> RuntimeResult<HostFd> {
        let h_fd = self.pop_fd()?;
        self.m[k] = Ok(h_fd);
        Ok(h_fd)
    }

    #[trusted]
    #[requires(k < MAX_SBOX_FDS)]
    #[ensures (self.lookup(k).is_err())]
    #[ensures (forall(|i: usize| (i < MAX_SBOX_FDS && i != k) ==>
                    self.lookup(i) == old(self.lookup(i))))]
    pub fn delete(&mut self, k: SboxFd) {
        if let Ok(oldfd) = self.m[k] {
            self.reserve.push(oldfd);
        }
        self.m[k] = Err(Ebadf);
    }
}
