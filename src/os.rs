use crate::types::*;
use prusti_contracts::*;
use syscall::syscall;

//TODO: prove path access safety

/// This module contains our syscall specifications
/// functions must be trusted because we don't know what the os actually does
/// on a syscall
/// VmCtx is included as an argument only to be used for preconditions

//TODO: pathname needs to be sandboxed
#[trusted]
pub fn os_open(ctx: &VmCtx, pathname: SandboxedPath, flags: i32) -> usize {
    // ACCESS_PATH(pathname);
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(OPEN, os_path.as_ptr(), flags) }
}

#[trusted]
pub fn os_close(ctx: &VmCtx, fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    return unsafe { syscall!(CLOSE, os_fd) };
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_read(ctx: &VmCtx, fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe {
        let result = syscall!(READ, os_fd, buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        return result;
    };
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_write(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    return unsafe { syscall!(WRITE, os_fd, buf.as_ptr(), cnt) };
}
