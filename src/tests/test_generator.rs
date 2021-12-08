use crate::os;
use crate::runtime::fresh_ctx;
use crate::tcb::os_specs::linux::os_read;
use crate::tcb::os_specs::linux::*;
use crate::types::{SboxPtr, VmCtx, LINEAR_MEM_SIZE};
use quickcheck::{QuickCheck, TestResult};
use quickcheck_macros;

pub fn ctx_safe(ctx: &VmCtx) -> bool {
    ctx.memlen == LINEAR_MEM_SIZE &&
    // ctx.mem.len() == LINEAR_MEM_SIZE &&
    ctx.argc < 1024 &&
    ctx.envc < 1024 &&
    ctx.arg_buffer.len() < 1024 * 1024 &&
    ctx.env_buffer.len() < 1024 * 1024
}

#[quickcheck_macros::quickcheck]
fn check_os_open(pathname: Vec<u8>, flags: i32) -> TestResult {
    let result = os_open(pathname, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_close(fd: usize) -> TestResult {
    let result = os_close(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_read(fd: usize, mut vec_buf: Vec<u8>, cnt: usize) -> TestResult {
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_read(fd, buf, cnt);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_pread(fd: usize, mut vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_pread(fd, buf, cnt, offset);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_write(fd: usize, vec_buf: Vec<u8>, cnt: usize) -> TestResult {
    let buf = &vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_write(fd, buf, cnt);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_pwrite(fd: usize, vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
    let buf = &vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_pwrite(fd, buf, cnt, offset);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_seek(fd: usize, offset: i64, whence: i32) -> TestResult {
    let result = os_seek(fd, offset, whence);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> TestResult {
    let result = os_advise(fd, offset, len, advice);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_allocate(fd: usize, offset: i64, len: i64) -> TestResult {
    let result = os_allocate(fd, offset, len);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_sync(fd: usize) -> TestResult {
    let result = os_sync(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_datasync(fd: usize) -> TestResult {
    let result = os_datasync(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
//#[quickcheck_macros::quickcheck]
//fn check_os_fstat(fd: usize, mut vec_stat: libc::stat) -> TestResult {
//    let stat = &mut vec_stat;
//    let result = os_fstat(fd, stat);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fstatat(fd: usize, path: Vec<u8>, mut vec_stat: libc::stat, flags: i32) -> TestResult {
//    let stat = &mut vec_stat;
//    let result = os_fstatat(fd, path, stat, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
#[quickcheck_macros::quickcheck]
fn check_os_fgetfl(fd: usize) -> TestResult {
    let result = os_fgetfl(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_fsetfl(fd: usize, flags: libc::c_int) -> TestResult {
    let result = os_fsetfl(fd, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_ftruncate(fd: usize, length: libc::off_t) -> TestResult {
    let result = os_ftruncate(fd, length);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_linkat(
    old_fd: usize,
    old_path: Vec<u8>,
    new_fd: usize,
    new_path: Vec<u8>,
    flags: i32,
) -> TestResult {
    let result = os_linkat(old_fd, old_path, new_fd, new_path, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> TestResult {
    let result = os_mkdirat(dir_fd, pathname, mode);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_readlinkat(
    dir_fd: usize,
    pathname: Vec<u8>,
    mut vec_buf: Vec<u8>,
    cnt: usize,
) -> TestResult {
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_readlinkat(dir_fd, pathname, buf, cnt);
    if !(!(result >= 0) || (buf.len() == result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> TestResult {
    let result = os_unlinkat(dir_fd, pathname, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_renameat(
    old_dir_fd: usize,
    old_pathname: Vec<u8>,
    new_dir_fd: usize,
    new_pathname: Vec<u8>,
) -> TestResult {
    let result = os_renameat(old_dir_fd, old_pathname, new_dir_fd, new_pathname);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_symlinkat(old_pathname: Vec<u8>, dir_fd: usize, new_pathname: Vec<u8>) -> TestResult {
    let result = os_symlinkat(old_pathname, dir_fd, new_pathname);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
//#[quickcheck_macros::quickcheck]
//fn check_os_futimens(fd: usize, vec_specs: Vec<libc::timespec>) -> TestResult {
//    let specs = &vec_specs;
//    if !(specs.capacity() >= 2) {
//        return TestResult::discard();
//    }
//    let result = os_futimens(fd, specs);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_utimensat(
//    fd: usize,
//    pathname: Vec<u8>,
//    vec_specs: Vec<libc::timespec>,
//    flags: libc::c_int,
//) -> TestResult {
//    let specs = &vec_specs;
//    if !(specs.capacity() >= 2) {
//        return TestResult::discard();
//    }
//    let result = os_utimensat(fd, pathname, specs, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_clock_get_time(clock_id: libc::clockid_t, mut vec_spec: libc::timespec) -> TestResult {
//    let spec = &mut vec_spec;
//    let result = os_clock_get_time(clock_id, spec);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_clock_get_res(clock_id: libc::clockid_t, mut vec_spec: libc::timespec) -> TestResult {
//    let spec = &mut vec_spec;
//    let result = os_clock_get_res(clock_id, spec);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
#[quickcheck_macros::quickcheck]
fn check_os_getrandom(mut vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_getrandom(buf, cnt, flags);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_recv(fd: usize, mut vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_recv(fd, buf, cnt, flags);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_send(fd: usize, vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
    let buf = &vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_send(fd, buf, cnt, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_shutdown(fd: usize, how: libc::c_int) -> TestResult {
    let result = os_shutdown(fd, how);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
//#[quickcheck_macros::quickcheck]
//fn check_os_nanosleep(vec_req: libc::timespec, mut vec_rem: libc::timespec) -> TestResult {
//    let req = &vec_req;
//    let rem = &mut vec_rem;
//    let result = os_nanosleep(req, rem);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_poll(mut vec_pollfd: libc::pollfd, timeout: libc::c_int) -> TestResult {
//    let pollfd = &mut vec_pollfd;
//    let result = os_poll(pollfd, timeout);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
#[quickcheck_macros::quickcheck]
fn check_os_getdents64(fd: usize, mut vec_dirp: Vec<u8>, count: usize) -> TestResult {
    let dirp = &mut vec_dirp;
    let result = os_getdents64(fd, dirp, count);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_socket(domain: i32, ty: i32, protocol: i32) -> TestResult {
    let result = os_socket(domain, ty, protocol);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
//#[quickcheck_macros::quickcheck]
//fn check_os_connect(sockfd: usize, vec_addr: libc::sockaddr_in, addrlen: u32) -> TestResult {
//    let addr = &vec_addr;
//    let result = os_connect(sockfd, addr, addrlen);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
#[quickcheck_macros::quickcheck]
fn check_memcpy_to_sandbox(dst: SboxPtr, vec_src: Vec<u8>, n: u32) -> TestResult {
    let src = &vec_src;
    let mut ctx = fresh_ctx(".".to_string());
    ctx.check_memcpy_to_sandbox_impl(dst, src, n)
}
impl VmCtx {
    fn check_memcpy_to_sandbox_impl(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) -> TestResult {
        if !(src.len() >= (n as usize)) {
            return TestResult::discard();
        }
        if !(self.fits_in_lin_mem(dst, n)) {
            return TestResult::discard();
        }
        if !(true && ctx_safe(self)) {
            return TestResult::discard();
        }
        let self_dot_memlen_old = self.memlen;
        let result = self.memcpy_to_sandbox(dst, src, n);
        if !(true && ctx_safe(self)) {
            return TestResult::failed();
        }
        if !(self_dot_memlen_old == self.memlen) {
            return TestResult::failed();
        }
        TestResult::passed()
    }
}
