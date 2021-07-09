use prusti_contracts::*;
use crate::types::*;
use syscall::syscall;

// #define ACCESS_MEM(ptr, size) memset(ptr, 0, size)  
// #define ACCESS_FD(fd) (assert(true)) 
// #define ACCESS_PATH(path) (assert(true)) 


//TODO: figure out how to best specify this

//TODO: { Path Sandboxing }
#[trusted]
pub fn os_open(pathname: *mut u8, flags: i32) -> isize {

    // ACCESS_PATH(pathname);
    unsafe {
        syscall!(OPEN, 
            pathname, 
            flags) as isize
    }
}

#[trusted]
pub fn os_close(fd: HostFd) -> i32 {
    // ACCESS_FD(fd);
    return unsafe {
        syscall!(CLOSE, fd) as i32
    };
}



#[trusted]
pub fn os_read(fd: HostFd, buf: *mut u8, cnt: usize) -> isize { 

    // ACCESS_MEM(buf, cnt); 
    // ACCESS_FD(fd); 
    return unsafe {
        syscall!(READ, 
            fd, 
            buf, 
            cnt) as isize
    }
}

//&[u8]
#[trusted]
pub fn os_write(fd: HostFd, buf: *mut u8, cnt: usize) -> isize{
    // ACCESS_MEM(buf, cnt);
    // ACCESS_FD(fd);
    return unsafe {
        syscall!(WRITE, 
            fd, 
            buf, 
            cnt) as isize
    }
}

