use crate::types::*;
use prusti_contracts::*;
use syscall::syscall;

//TODO: prove fd access safety
//TODO: prove path access safety

/// This module contains our syscall specifications
/// functions must be trusted because we don't know what the os actually does
/// on a syscall
/// VmCtx is included as an argument only to be used for preconditions

//TODO: pathname needs to be sandboxed
#[trusted]
pub fn os_open(ctx: &VmCtx, pathname: &mut Vec<u8>, flags: i32) -> isize {
    // ACCESS_PATH(pathname);
    unsafe { syscall!(OPEN, pathname.as_mut_ptr(), flags) as isize }
}

//TODO
#[trusted]
pub fn os_close(ctx: &VmCtx, fd: HostFd) -> i32 {
    // ACCESS_FD(fd);
    return unsafe { syscall!(CLOSE, fd) as i32 };
}

//TODO: move set_len outside syscall (should also probably use result)
//TODO: FD safety
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == cnt)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_read(ctx: &VmCtx, fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> isize {
    // ACCESS_FD(fd);
    unsafe {
        let result = syscall!(READ, fd, buf.as_mut_ptr(), cnt) as isize;
        buf.set_len(cnt);
        return result;
    };
}

//TODO: fd safety
//TODO: check all input buffers have a sufficient amount of reserved space
#[trusted]
pub fn os_write(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize) -> isize {
    // ACCESS_FD(fd);
    return unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) as isize };
}
