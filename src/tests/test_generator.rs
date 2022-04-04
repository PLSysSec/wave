use crate::os;
use crate::runtime::fresh_ctx;
use crate::tcb::os_specs::os_read;
use crate::tcb::os_specs::*;
use crate::tcb::verifier::*;
use crate::tests::init;
use crate::types::{SboxPtr, VmCtx, LINEAR_MEM_SIZE};
use quickcheck::{QuickCheck, TestResult};
use quickcheck_macros;

#[quickcheck_macros::quickcheck]
fn check_os_pread(fd: usize, mut vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
    init();
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
//#[quickcheck_macros::quickcheck]
//fn check_os_pwrite(fd: usize, vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
//    init();
//    let buf = &vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_pwrite(fd, buf, cnt, offset);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> TestResult {
//    init();
//    let result = os_advise(fd, offset, len, advice);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_allocate(fd: usize, offset: i64, len: i64) -> TestResult {
//    init();
//    let result = os_allocate(fd, offset, len);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fstatat(fd: usize, path: Vec<u8>, mut vec_stat: libc::stat, flags: i32) -> TestResult {
//    init();
//    let stat = &mut vec_stat;
//    let result = os_fstatat(fd, path, stat, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_futimens(fd: usize, vec_specs: Vec<libc::timespec>) -> TestResult {
//    init();
//    let specs = &vec_specs;
//    if !(specs.len() >= 2) {
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
//    init();
//    let specs = &vec_specs;
//    if !(specs.len() >= 2) {
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
//    init();
//    let spec = &mut vec_spec;
//    let result = os_clock_get_time(clock_id, spec);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_clock_get_res(clock_id: libc::clockid_t, mut vec_spec: libc::timespec) -> TestResult {
//    init();
//    let spec = &mut vec_spec;
//    let result = os_clock_get_res(clock_id, spec);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_getrandom(mut vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
//    init();
//    let buf = &mut vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_getrandom(buf, cnt, flags);
//    if !(!(result >= 0) || (buf.len() >= result as usize)) {
//        return TestResult::failed();
//    }
//    if !(!(result >= 0) || (result as usize <= cnt)) {
//        return TestResult::failed();
//    }
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_nanosleep(vec_req: libc::timespec, mut vec_rem: libc::timespec) -> TestResult {
//    init();
//    let req = &vec_req;
//    let rem = &mut vec_rem;
//    let result = os_nanosleep(req, rem);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_getdents64(fd: usize, mut vec_dirp: Vec<u8>, count: usize) -> TestResult {
//    init();
//    let dirp = &mut vec_dirp;
//    if !(dirp.capacity() >= count) {
//        return TestResult::discard();
//    }
//    let result = os_getdents64(fd, dirp, count);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fstat(fd: usize, mut vec_stat: libc::stat) -> TestResult {
//    init();
//    let stat = &mut vec_stat;
//    let result = os_fstat(fd, stat);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_pread(fd: usize, mut vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
//    init();
//    let buf = &mut vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_pread(fd, buf, cnt, offset);
//    if !(!(result >= 0) || (buf.len() >= result as usize)) {
//        return TestResult::failed();
//    }
//    if !(!(result >= 0) || (result as usize <= cnt)) {
//        return TestResult::failed();
//    }
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_pwrite(fd: usize, vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
//    init();
//    let buf = &vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_pwrite(fd, buf, cnt, offset);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_allocate(fd: usize, vec_fstore: libc::fstore_t) -> TestResult {
//    init();
//    let fstore = &vec_fstore;
//    let result = os_allocate(fd, fstore);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fstatat(fd: usize, path: Vec<u8>, mut vec_stat: libc::stat, flags: i32) -> TestResult {
//    init();
//    let stat = &mut vec_stat;
//    let result = os_fstatat(fd, path, stat, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_futimens(fd: usize, vec_specs: Vec<libc::timespec>) -> TestResult {
//    init();
//    let specs = &vec_specs;
//    if !(specs.len() >= 2) {
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
//    init();
//    let specs = &vec_specs;
//    if !(specs.len() >= 2) {
//        return TestResult::discard();
//    }
//    let result = os_utimensat(fd, pathname, specs, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_gettimeofday(mut vec_timeval: libc::timeval) -> TestResult {
//    init();
//    let timeval = &mut vec_timeval;
//    let result = os_gettimeofday(timeval);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_getboottime(mut vec_timeval: libc::timeval) -> TestResult {
//    init();
//    let timeval = &mut vec_timeval;
//    let result = os_getboottime(timeval);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_rusageself(mut vec_rusage: libc::rusage) -> TestResult {
//    init();
//    let rusage = &mut vec_rusage;
//    let result = os_rusageself(rusage);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_thread_selfusage() -> TestResult {
//    init();
//    let result = os_thread_selfusage();
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_getrandom(mut vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
//    init();
//    let buf = &mut vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_getrandom(buf, cnt, flags);
//    if !(!(result >= 0) || (buf.len() >= result as usize)) {
//        return TestResult::failed();
//    }
//    if !(!(result >= 0) || (result as usize <= cnt)) {
//        return TestResult::failed();
//    }
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_wait_until(deadline: u64) -> TestResult {
//    init();
//    let result = os_wait_until(deadline);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_timebase_info(mut vec_info: mach_timebase_info_data_t) -> TestResult {
//    init();
//    let info = &mut vec_info;
//    let result = os_timebase_info(info);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_getdents64(fd: usize, mut vec_dirp: Vec<u8>, count: usize) -> TestResult {
//    init();
//    let dirp = &mut vec_dirp;
//    if !(dirp.capacity() >= count) {
//        return TestResult::discard();
//    }
//    let result = os_getdents64(fd, dirp, count);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fstat(fd: usize, mut vec_stat: libc::stat) -> TestResult {
//    init();
//    let stat = &mut vec_stat;
//    let result = os_fstat(fd, stat);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_openat(dirfd: usize, pathname: Vec<u8>, flags: i32) -> TestResult {
//    init();
//    let result = os_openat(dirfd, pathname, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_close(fd: usize) -> TestResult {
//    init();
//    let result = os_close(fd);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_read(fd: usize, mut vec_buf: Vec<u8>, cnt: usize) -> TestResult {
//    init();
//    let buf = &mut vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_read(fd, buf, cnt);
//    if !(!(result >= 0) || (buf.len() >= result as usize)) {
//        return TestResult::failed();
//    }
//    if !(!(result >= 0) || (result as usize <= cnt)) {
//        return TestResult::failed();
//    }
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_write(fd: usize, vec_buf: Vec<u8>, cnt: usize) -> TestResult {
//    init();
//    let buf = &vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_write(fd, buf, cnt);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_seek(fd: usize, offset: i64, whence: i32) -> TestResult {
//    init();
//    let result = os_seek(fd, offset, whence);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_sync(fd: usize) -> TestResult {
//    init();
//    let result = os_sync(fd);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_datasync(fd: usize) -> TestResult {
//    init();
//    let result = os_datasync(fd);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fgetfl(fd: usize) -> TestResult {
//    init();
//    let result = os_fgetfl(fd);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fsetfl(fd: usize, flags: libc::c_int) -> TestResult {
//    init();
//    let result = os_fsetfl(fd, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_ftruncate(fd: usize, length: libc::off_t) -> TestResult {
//    init();
//    let result = os_ftruncate(fd, length);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_linkat(
//    old_fd: usize,
//    old_path: Vec<u8>,
//    new_fd: usize,
//    new_path: Vec<u8>,
//    flags: i32,
//) -> TestResult {
//    init();
//    let result = os_linkat(old_fd, old_path, new_fd, new_path, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> TestResult {
//    init();
//    let result = os_mkdirat(dir_fd, pathname, mode);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_readlinkat(
//    dir_fd: usize,
//    pathname: Vec<u8>,
//    mut vec_buf: Vec<u8>,
//    cnt: usize,
//) -> TestResult {
//    init();
//    let buf = &mut vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_readlinkat(dir_fd, pathname, buf, cnt);
//    if !(!(result >= 0) || (buf.len() == result as usize)) {
//        return TestResult::failed();
//    }
//    if !(!(result >= 0) || (result as usize <= cnt)) {
//        return TestResult::failed();
//    }
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> TestResult {
//    init();
//    let result = os_unlinkat(dir_fd, pathname, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_renameat(
//    old_dir_fd: usize,
//    old_pathname: Vec<u8>,
//    new_dir_fd: usize,
//    new_pathname: Vec<u8>,
//) -> TestResult {
//    init();
//    let result = os_renameat(old_dir_fd, old_pathname, new_dir_fd, new_pathname);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_symlinkat(old_pathname: Vec<u8>, dir_fd: usize, new_pathname: Vec<u8>) -> TestResult {
//    init();
//    let result = os_symlinkat(old_pathname, dir_fd, new_pathname);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_recv(fd: usize, mut vec_buf: Vec<u8>, cnt: usize, flags: i32) -> TestResult {
//    init();
//    let buf = &mut vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_recv(fd, buf, cnt, flags);
//    if !(!(result >= 0) || (buf.len() >= result as usize)) {
//        return TestResult::failed();
//    }
//    if !(!(result >= 0) || (result as usize <= cnt)) {
//        return TestResult::failed();
//    }
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_send(fd: usize, vec_buf: Vec<u8>, cnt: usize, flags: i32) -> TestResult {
//    init();
//    let buf = &vec_buf[..];
//    if !(buf.len() >= cnt) {
//        return TestResult::discard();
//    }
//    let result = os_send(fd, buf, cnt, flags);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_shutdown(fd: usize, how: libc::c_int) -> TestResult {
//    init();
//    let result = os_shutdown(fd, how);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_poll(mut vec_pollfds: Vec<libc::pollfd>, timeout: libc::c_int) -> TestResult {
//    init();
//    let pollfds = &mut vec_pollfds[..];
//    let result = os_poll(pollfds, timeout);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_socket(domain: i32, ty: i32, protocol: i32) -> TestResult {
//    init();
//    let result = os_socket(domain, ty, protocol);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_connect(sockfd: usize, vec_addr: libc::sockaddr_in, addrlen: u32) -> TestResult {
//    init();
//    let addr = &vec_addr;
//    let result = os_connect(sockfd, addr, addrlen);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_os_fionread(fd: usize) -> TestResult {
//    init();
//    let result = os_fionread(fd);
//    if !(true) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_as_sbox_ptr(vec_slice: Vec<u8>) -> TestResult {
//    init();
//    let slice = &vec_slice[..];
//    let result = as_sbox_ptr(slice);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_get_components(vec_path: PathBuf) -> TestResult {
//    init();
//    let path = &vec_path;
//    let result = get_components(path);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_addr_matches_netlist_entry(
//    vec_netlist: Netlist,
//    addr: u32,
//    port: u32,
//    idx: usize,
//) -> TestResult {
//    init();
//    let netlist = &vec_netlist;
//    if !(idx < 4) {
//        return TestResult::discard();
//    }
//    let result = addr_matches_netlist_entry(netlist, addr, port, idx);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_normalize_path(vec_path: PathBuf) -> TestResult {
//    init();
//    let path = &vec_path;
//    let result = normalize_path(path);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_vec_checked_lookup(vec_vec: Vec<RuntimeResult<HostFd>>, index: SboxFd) -> TestResult {
//    init();
//    let vec = &vec_vec;
//    if !(index < MAX_SBOX_FDS) {
//        return TestResult::discard();
//    }
//    let result = vec_checked_lookup(vec, index);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_nth_bit_set(bv: u16, n: i32) -> TestResult {
//    init();
//    let result = nth_bit_set(bv, n);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_nth_bit_set_u32(bv: u32, n: u32) -> TestResult {
//    init();
//    let result = nth_bit_set_u32(bv, n);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_with_nth_bit_set(bv: u16, n: i32) -> TestResult {
//    init();
//    let result = with_nth_bit_set(bv, n);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_bitwise_and(bv1: i32, bv2: i32) -> TestResult {
//    init();
//    let result = bitwise_and(bv1, bv2);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_bitwise_and_i16(bv1: i16, bv2: i16) -> TestResult {
//    init();
//    let result = bitwise_and_i16(bv1, bv2);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_bitwise_and_u16(bv1: u16, bv2: u16) -> TestResult {
//    init();
//    let result = bitwise_and_u16(bv1, bv2);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_bitwise_and_u32(bv1: u32, bv2: u32) -> TestResult {
//    init();
//    let result = bitwise_and_u32(bv1, bv2);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_bitwise_and_u64(bv1: u64, bv2: u64) -> TestResult {
//    init();
//    let result = bitwise_and_u64(bv1, bv2);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_bitwise_or(bv1: i32, bv2: i32) -> TestResult {
//    init();
//    let result = bitwise_or(bv1, bv2);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_bitwise_or_u32(bv1: u32, bv2: u32) -> TestResult {
//    init();
//    let result = bitwise_or_u32(bv1, bv2);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_fresh_stat() -> TestResult {
//    init();
//    let result = fresh_stat();
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_fresh_rusage() -> TestResult {
//    init();
//    let result = fresh_rusage();
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_first_null(vec_buf: Vec<u8>, start: usize, offset: usize, len: usize) -> TestResult {
//    init();
//    let buf = &vec_buf;
//    if !(len >= offset) {
//        return TestResult::discard();
//    }
//    if !(buf.len() >= start + len) {
//        return TestResult::discard();
//    }
//    let len_old = len;
//    let result = first_null(buf, start, offset, len);
//    if !(result < len_old) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_push_dirent_name(
//    mut vec_out_buf: Vec<u8>,
//    vec_buf: Vec<u8>,
//    start: usize,
//    len: usize,
//) -> TestResult {
//    init();
//    let out_buf = &mut vec_out_buf;
//    let buf = &vec_buf;
//    let result = push_dirent_name(out_buf, buf, start, len);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_clone_vec_u8(vec_vec: Vec<u8>) -> TestResult {
//    init();
//    let vec = &vec_vec;
//    let vec_dot_len_method_old = vec.len();
//    let result = clone_vec_u8(vec);
//    if !(result.len() == vec_dot_len_method_old) {
//        return TestResult::failed();
//    }
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_get_homedir_fd(vec_s: String) -> TestResult {
//    init();
//    let s = &vec_s;
//    let result = get_homedir_fd(s);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_string_to_vec_u8(vec_s: String) -> TestResult {
//    init();
//    let s = &vec_s;
//    let result = string_to_vec_u8(s);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_empty_netlist() -> TestResult {
//    init();
//    let result = empty_netlist();
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_as_u32(e: RuntimeError) -> TestResult {
//    init();
//    let result = as_u32(e);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_as_u16(e: RuntimeError) -> TestResult {
//    init();
//    let result = as_u16(e);
//    TestResult::passed()
//}
//#[quickcheck_macros::quickcheck]
//fn check_memcpy_from_sandbox(mut vec_dst: Vec<u8>, src: SboxPtr, n: u32) -> TestResult {
//    init();
//    let dst = &mut vec_dst;
//    let mut ctx = fresh_ctx(".".to_string());
//    ctx.check_memcpy_from_sandbox_impl(dst, src, n)
//}
//#[quickcheck_macros::quickcheck]
//fn check_memcpy_to_sandbox(dst: SboxPtr, vec_src: Vec<u8>, n: u32) -> TestResult {
//    init();
//    let src = &vec_src;
//    let mut ctx = fresh_ctx(".".to_string());
//    ctx.check_memcpy_to_sandbox_impl(dst, src, n)
//}
//#[quickcheck_macros::quickcheck]
//fn check_slice_mem_mut(ptr: SboxPtr, len: u32) -> TestResult {
//    init();
//    let mut ctx = fresh_ctx(".".to_string());
//    ctx.check_slice_mem_mut_impl(ptr, len)
//}
//#[quickcheck_macros::quickcheck]
//fn check_resolve_path(in_path: Vec<u8>) -> TestResult {
//    init();
//    let mut ctx = fresh_ctx(".".to_string());
//    ctx.check_resolve_path_impl(in_path)
//}
//impl VmCtx {
//    fn check_memcpy_from_sandbox_impl(
//        &self,
//        dst: &mut Vec<u8>,
//        src: SboxPtr,
//        n: u32,
//    ) -> TestResult {
//        if !(dst.capacity() >= (n as usize)) {
//            return TestResult::discard();
//        }
//        if !(self.fits_in_lin_mem(src, n)) {
//            return TestResult::discard();
//        }
//        if !(ctx_safe(self)) {
//            return TestResult::discard();
//        }
//        if !(true) {
//            return TestResult::discard();
//        }
//        let result = self.memcpy_from_sandbox(dst, src, n);
//        if !(dst.len() == (n as usize)) {
//            return TestResult::failed();
//        }
//        if !(true) {
//            return TestResult::failed();
//        }
//        TestResult::passed()
//    }
//    fn check_memcpy_to_sandbox_impl(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) -> TestResult {
//        if !(src.len() >= (n as usize)) {
//            return TestResult::discard();
//        }
//        if !(self.fits_in_lin_mem(dst, n)) {
//            return TestResult::discard();
//        }
//        if !(ctx_safe(self)) {
//            return TestResult::discard();
//        }
//        if !(true) {
//            return TestResult::discard();
//        }
//        let result = self.memcpy_to_sandbox(dst, src, n);
//        if !(ctx_safe(self)) {
//            return TestResult::failed();
//        }
//        if !(true) {
//            return TestResult::failed();
//        }
//        if !(true) {
//            return TestResult::failed();
//        }
//        TestResult::passed()
//    }
//    fn check_slice_mem_mut_impl(&mut self, ptr: SboxPtr, len: u32) -> TestResult {
//        if !(self.fits_in_lin_mem(ptr, len)) {
//            return TestResult::discard();
//        }
//        if !(true) {
//            return TestResult::discard();
//        }
//        let ptr_as_usize_old = ptr as usize;
//        let result = self.slice_mem_mut(ptr, len);
//        if !(result.len() == (len as usize)) {
//            return TestResult::failed();
//        }
//        if !(true) {
//            return TestResult::failed();
//        }
//        if !(as_sbox_ptr(result) == ptr_as_usize_old) {
//            return TestResult::failed();
//        }
//        TestResult::passed()
//    }
//    fn check_resolve_path_impl(&self, in_path: Vec<u8>) -> TestResult {
//        let result = self.resolve_path(in_path);
//        TestResult::passed()
//    }
//}
