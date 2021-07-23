use crate::types::*;
use prusti_contracts::*;
use syscall::syscall;

// #define ACCESS_MEM(ptr, size) memset(ptr, 0, size)
// #define ACCESS_FD(fd) (assert(true))
// #define ACCESS_PATH(path) (assert(true))

//TODO: figure out how to best specify this

//TODO: { Path Sandboxing }
//TODO: first arg should be pathname?
#[trusted]
pub fn os_open(pathname: &mut Vec<u8>, flags: i32) -> isize {
    // ACCESS_PATH(pathname);
    unsafe { syscall!(OPEN, pathname.as_mut_ptr(), flags) as isize }
}

#[trusted]
pub fn os_close(fd: HostFd) -> i32 {
    // ACCESS_FD(fd);
    return unsafe { syscall!(CLOSE, fd) as i32 };
}

//TODO: need to set length of buf
//TODO: check all input buffers have a sufficient amount of reserved space
#[trusted]
pub fn os_read(fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> isize {
    // ACCESS_MEM(buf, cnt);
    // ACCESS_FD(fd);
    return unsafe { syscall!(READ, fd, buf.as_mut_ptr(), cnt) as isize };
}

//&[u8]
#[trusted]
pub fn os_write(fd: HostFd, buf: &Vec<u8>, cnt: usize) -> isize {
    // ACCESS_MEM(buf, cnt);
    // ACCESS_FD(fd);
    // ctx.mem.as_ptr().wrapping_add(v_buf),
    return unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) as isize };
}
