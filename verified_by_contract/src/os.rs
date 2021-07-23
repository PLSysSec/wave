use crate::types::*;
use prusti_contracts::*;
use syscall::syscall;

// #define ACCESS_MEM(ptr, size) memset(ptr, 0, size)
// #define ACCESS_FD(fd) (assert(true))
// #define ACCESS_PATH(path) (assert(true))


/// VmCtx is included as an argument only to be used for preconditions

//TODO: figure out how to best specify this
 
//TODO: { Path Sandboxing }
//TODO: first arg should be pathname?
#[trusted]
pub fn os_open(ctx: &VmCtx, pathname: &mut Vec<u8>, flags: i32) -> isize {
    // ACCESS_PATH(pathname);
    unsafe { syscall!(OPEN, pathname.as_mut_ptr(), flags) as isize }
}

#[trusted]
pub fn os_close(ctx: &VmCtx, fd: HostFd) -> i32 {
    // ACCESS_FD(fd);
    return unsafe { syscall!(CLOSE, fd) as i32 };
}


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

//TODO: need to set length of buf
//TODO: check all input buffers have a sufficient amount of reserved space
#[trusted]
pub fn os_write(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize) -> isize {
    // ACCESS_FD(fd);
    return unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) as isize };
}
