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
    unsafe { syscall!(CLOSE, os_fd) }
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
        result
    }
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_pread(fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe {
        let result = syscall!(PREAD64, os_fd, buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        result
    }
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_write(fd: HostFd, buf: &Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(WRITE, os_fd, buf.as_ptr(), cnt) }
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_pwrite(fd: HostFd, buf: &Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(WRITE, os_fd, buf.as_ptr(), cnt) }
}

// TODO: could be cleaner to do a typedef SyscallRet = usize or something for From traits
#[trusted]
pub fn os_seek(fd: HostFd, offset: i64, whence: i32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(LSEEK, os_fd, offset, whence) }
}

#[trusted]
pub fn os_advise(fd: HostFd, offset: i64, len: i64, advice: i32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FADVISE64, os_fd, offset, len, advice) }
}

#[trusted]
pub fn os_allocate(fd: HostFd, offset: i64, len: i64) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FALLOCATE, os_fd, offset, len) }
}

#[trusted]
pub fn os_sync(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FSYNC, os_fd) }
}

#[trusted]
pub fn os_datasync(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FDATASYNC, os_fd) }
}

#[trusted]
pub fn os_fstat(fd: HostFd, stat: &mut libc::stat) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FSTAT, os_fd, stat as *mut libc::stat) }
}

#[trusted]
pub fn os_fgetfl(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FCNTL, os_fd, libc::F_GETFL, 0) }
}

#[trusted]
pub fn os_ftruncate(fd: HostFd, length: libc::off_t) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FTRUNCATE, os_fd, length) }
}

#[trusted]
pub fn os_clock_get_time(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETTIME, clock_id, spec as *mut libc::timespec) }
}

#[trusted]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETRES, clock_id, spec as *mut libc::timespec) }
}
