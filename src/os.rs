use crate::types::*;
use prusti_contracts::*;
use syscall::syscall;

/// This module contains our syscall specifications
/// These functions must be trusted because we don't know what the os actually does
/// on a syscall

#[trusted]
pub fn os_open(pathname: SandboxedPath, flags: i32) -> usize {
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(OPEN, os_path.as_ptr(), flags) }
}

#[trusted]
pub fn os_close(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    return unsafe { syscall!(CLOSE, os_fd) };
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_read(fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe {
        let result = syscall!(READ, os_fd, buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        return result;
    };
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_write(fd: HostFd, buf: &Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    return unsafe { syscall!(WRITE, os_fd, buf.as_ptr(), cnt) };
}

#[trusted]
pub fn os_seek(fd: HostFd, offset: i64, whence: i32) -> usize {
    let os_fd: usize = fd.into();
    return unsafe { syscall!(LSEEK, os_fd, offset, whence) };
}

#[trusted]
pub fn os_sync(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FSYNC, os_fd) }
}
