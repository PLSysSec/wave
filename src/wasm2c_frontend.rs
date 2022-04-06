use crate::tcb::ffi::*;
use crate::types::*;
use crate::wrappers::*;
use crate::writeback::*;
use libc::{c_char, strlen};
use std::ffi::CStr;
use std::os::unix::io::AsRawFd;
use std::time::Instant;
use trace::trace;
use RuntimeError::*;
// use log::{debug, error, log_enabled, info, Level};
use env_logger;
use log;

// When we are not timing syscalls, disable time syscalls
// TODO: clean this up somehow
#[cfg(not(feature = "time_hostcalls"))]
use crate::stats::noop_instrumentation::{push_hostcall_result, start_timer, stop_timer};
#[cfg(feature = "time_hostcalls")]
use crate::stats::timing::{push_hostcall_result, start_timer, stop_timer};

#[cfg(not(feature = "time_hostcalls"))]
use crate::stats::noop_instrumentation::output_hostcall_perf_results;
#[cfg(not(feature = "time_syscalls"))]
use crate::stats::noop_instrumentation::output_syscall_perf_results;
#[cfg(feature = "time_hostcalls")]
use crate::stats::stats::output_hostcall_perf_results;
#[cfg(feature = "time_syscalls")]
use crate::stats::stats::output_syscall_perf_results;

trace::init_depth_var!();

pub fn create_ctx(
    memptr: *mut u8,
    homedir: &str,
    mut arg_buffer: Vec<u8>,
    argc: usize,
    mut env_buffer: Vec<u8>,
    envc: usize,
    netlist: Netlist,
) -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    let mut fdmap = FdMap::new();
    fdmap.init_std_fds();
    let homedir_file = std::fs::File::open(homedir).unwrap();
    let homedir_host_fd = homedir_file.as_raw_fd() as usize;
    if homedir_host_fd >= 0 {
        fdmap.create(HostFd::from_raw(homedir_host_fd));
    }
    // Need to forget file to make sure it does not get auto-closed
    // when it gets out of scope
    std::mem::forget(homedir_file);
    // replace all space with null.
    // This makes it easy to return the arg_buffer later
    for i in 0..arg_buffer.len() {
        if arg_buffer[i] == b' ' {
            arg_buffer[i] = b'\0';
        }
    }

    // replace all space with null.
    // This makes it easy to return the env_buffer later
    for i in 0..env_buffer.len() {
        if env_buffer[i] == b' ' {
            env_buffer[i] = b'\0';
        }
    }

    let mem = ffi_load_vec(memptr, memlen);

    VmCtx {
        mem,
        memlen,
        fdmap,
        homedir: homedir.to_owned(),
        homedir_host_fd: HostFd::from_raw(homedir_host_fd),
        arg_buffer,
        argc,
        env_buffer,
        envc,
        netlist,
    }
}

/// Used for FFI. (wasm2c frontend)
/// Initialize a vmctx with a memory that points to memptr
/// TODO: depulicate with fresh_ctx()
/// TODO: clean up this function, make some helpers, etc
/// scary
fn ctx_from_memptr(
    memptr: *mut u8,
    memsize: isize,
    homedir: *const c_char,
    args: *mut u8,
    argc: usize,
    env: *mut u8,
    envc: usize,
    log_path: *mut c_char,
    netlist: *const Netlist,
) -> VmCtx {
    let netlist = transmut_netlist(netlist);
    let log_path = ffi_load_cstr(log_path).to_owned().clone();
    let homedir = &(*ffi_load_cstr(homedir).clone()); // Actually copy the inner string

    let arg_buffer = ffi_load_cstr_as_vec(args).clone();
    let env_buffer = ffi_load_cstr_as_vec(env).clone();

    create_ctx(memptr, homedir, arg_buffer, argc, env_buffer, envc, netlist)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn wave_init(
    memptr: *mut u8,
    memsize: isize,
    homedir: *const c_char,
    args: *mut u8,
    argc: usize,
    env: *mut u8,
    envc: usize,
    log_path: *mut c_char,
    netlist: *const Netlist,
) -> *mut VmCtx {
    env_logger::init(); // removing this line kills tracing
    let ctx = ctx_from_memptr(
        memptr, memsize, homedir, args, argc, env, envc, log_path, netlist,
    );
    // convert the ctx into a raw pointer for the runtime
    // must manually destruct later
    Box::into_raw(Box::new(ctx))
}

#[no_mangle]
pub extern "C" fn wave_cleanup(ctx: *const *mut VmCtx) {
    output_hostcall_perf_results();
    output_syscall_perf_results();
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_args_getZ_iii(
    ctx: *const *mut VmCtx,
    argv: u32,
    argv_buf: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_args_get(ctx_ref, argv, argv_buf);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("args_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_args_sizes_getZ_iii(
    ctx: *const *mut VmCtx,
    pargc: u32,
    pargv_buf_size: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_args_sizes_get(ctx_ref);
    let retval =
        wasm2c_marshal_and_writeback_u32_pair(ctx_ref, pargc as usize, pargv_buf_size as usize, r);
    let end = stop_timer();
    push_hostcall_result("args_sizes_get", start, end);
    retval
}

// TODO: this needs to invoke the cleanup function
#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_proc_exitZ_vi(ctx: *const *mut VmCtx, x: u32) {
    std::process::exit(x as i32);
    // let start = start_timer();
    //let ctx_ref = ptr_to_ref(ctx);
    // wasi_proc_exit(ctx_ref, x);
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_sizes_getZ_iii(
    ctx: *const *mut VmCtx,
    pcount: u32,
    pbuf_size: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_environ_sizes_get(ctx_ref);
    let retval =
        wasm2c_marshal_and_writeback_u32_pair(ctx_ref, pcount as usize, pbuf_size as usize, r);
    let end = stop_timer();
    push_hostcall_result("environ_sizes_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_getZ_iii(
    ctx: *const *mut VmCtx,
    __environ: u32,
    environ_buf: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_environ_get(ctx_ref, __environ, environ_buf);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("environ_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_prestat_getZ_iii(
    ctx: *const *mut VmCtx,
    fd: u32,
    prestat: u32,
) -> u32 {
    // Wasm2c implementation
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_prestat_get(ctx_ref, fd);
    let retval = wasm2c_marshal_and_writeback_prestat(ctx_ref, prestat as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_prestat_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_writeZ_iiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    iov: u32,
    iovcnt: u32,
    pnum: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_write(ctx_ref, fd, iov, iovcnt);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, pnum as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_write", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_readZ_iiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    iov: u32,
    iovcnt: u32,
    pnum: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_read(ctx_ref, fd, iov, iovcnt);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, pnum as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_read", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_closeZ_ii(ctx: *const *mut VmCtx, fd: u32) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_close(ctx_ref, fd);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_close", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_seekZ_iijii(
    ctx: *const *mut VmCtx,
    fd: u32,
    offset: u64,
    whence: u32,
    new_offset: u32, // output
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_seek(ctx_ref, fd, offset as i64, whence);
    let retval = wasm2c_marshal_and_writeback_u64(ctx_ref, new_offset as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_seek", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_time_getZ_iiji(
    ctx: *const *mut VmCtx,
    clock_id: u32,
    precision: u64,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_clock_time_get(ctx_ref, clock_id, 0);
    let retval = wasm2c_marshal_and_writeback_timestamp(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("clock_time_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_res_getZ_iii(
    ctx: *const *mut VmCtx,
    clock_id: u32,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_clock_res_get(ctx_ref, clock_id);
    let retval = wasm2c_marshal_and_writeback_timestamp(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("clock_res_get", start, end);
    retval
}

// void wasm_rt_sys_init() {
// void wasm_rt_init_wasi(wasm_sandbox_wasi_data* wasi_data) {
// void wasm_rt_cleanup_wasi(wasm_sandbox_wasi_data* wasi_data) {

/*
Wasi API that is not currently supported by wasm2c
*/

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_adviseZ_iijji(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    offset: u64,
    len: u64,
    advice: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_advise(ctx_ref, v_fd, offset, len, advice);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_advise", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_allocateZ_iijj(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    offset: u64,
    len: u64,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_allocate(ctx_ref, v_fd, offset, len);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_allocate", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_datasyncZ_ii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_datasync(ctx_ref, v_fd);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_datasync", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_fdstat_getZ_iii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_fdstat_get(ctx_ref, v_fd);
    let retval = wasm2c_marshal_and_writeback_fdstat(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_fdstat_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_fdstat_set_flagsZ_iii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    flags: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_fdstat_set_flags(ctx_ref, v_fd, flags);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_filestat_set_flags", start, end);
    retval
}

// Not supporting this because rights are getting removed
#[no_mangle]
#[trace(logging)]
pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_fdstat_set_rightsZ_iijj(
    ctx: *const *mut VmCtx,
    a: u32,
    b: u64,
    c: u64,
) -> u32 {
    unimplemented!()
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_filestat_getZ_iii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_filestat_get(ctx_ref, v_fd);
    let retval = wasm2c_marshal_and_writeback_filestat(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_filestat_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_filestat_set_sizeZ_iij(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    size: u64,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_filestat_set_size(ctx_ref, v_fd, size as i64);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_filestat_set_size", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_filestat_set_timesZ_iijji(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    atim: u64,
    mtim: u64,
    fst_flags: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_filestat_set_times(ctx_ref, v_fd, atim, mtim, fst_flags);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_filestat_set_times", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_preadZ_iiiiji(
    ctx: *const *mut VmCtx,
    fd: u32,
    iovs: u32,
    iov_len: u32,
    offset: u64,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_pread(ctx_ref, fd, iovs, iov_len, offset);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_pread", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_prestat_dir_nameZ_iiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    path: u32,
    path_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_prestat_dirname(ctx_ref, fd, path, path_len);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_prestat_dir_name", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_pwriteZ_iiiiji(
    ctx: *const *mut VmCtx,
    fd: u32,
    iovs: u32,
    iov_len: u32,
    offset: u64,
    retptr: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_pwrite(ctx_ref, fd, iovs, iov_len, offset);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_pwrite", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_readdirZ_iiiiji(
    ctx: *const *mut VmCtx,
    fd: u32,
    buf: u32,
    buf_len: u32,
    cookie: u64, // ???
    retptr: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_readdir(ctx_ref, fd, buf, buf_len as usize, cookie);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_readdir", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_renumberZ_iii(
    ctx: *const *mut VmCtx,
    from: u32,
    to: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_renumber(ctx_ref, from, to);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_renumber", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_syncZ_ii(ctx: *const *mut VmCtx, fd: u32) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_sync(ctx_ref, fd);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("fd_sync", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_tellZ_iii(
    ctx: *const *mut VmCtx,
    fd: u32,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_tell(ctx_ref, fd);
    let retval = wasm2c_marshal_and_writeback_u64(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("fd_tell", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_create_directoryZ_iiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    pathname: u32,
    path_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_create_directory(ctx_ref, fd, pathname, path_len);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("path_create_directory", start, end);
    retval
}

// wasi libc truncates result to 16 bits ???
#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_filestat_getZ_iiiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    flags: u32,
    path: u32,
    path_len: u32,
    out: u32, // wasm2c and wasi-libc disagree about 4 vs 5 arguments
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_filestat_get(ctx_ref, fd, flags, path, path_len);
    let retval = wasm2c_marshal_and_writeback_filestat(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("path_filestat_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_filestat_set_timesZ_iiiiijji(
    ctx: *const *mut VmCtx,
    fd: u32,
    flags: u32,
    path: u32,
    path_len: u32, // wasi-libc and wasm2c disagree about whether this arg should exist
    atim: u64,
    mtim: u64,
    fst_flags: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_filestat_set_times(ctx_ref, fd, flags, path, path_len, atim, mtim, fst_flags);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("path_filestat_set_times", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_linkZ_iiiiiiii(
    ctx: *const *mut VmCtx,
    old_fd: u32,
    old_flags: u32,
    old_path: u32,
    old_path_len: u32,
    new_fd: u32,
    new_path: u32,
    new_path_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_link(
        ctx_ref,
        old_fd,
        old_flags,
        old_path,
        old_path_len,
        new_fd,
        new_path,
        new_path_len,
    );
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("path_link", start, end);
    retval
}

fn adjust_oflags(oflags: u32, fs_rights_base: u64) -> u32 {
    if fs_rights_base & (1 << 6) != 0 {
        // can_write
        if fs_rights_base & (1 << 1) != 0 {
            // can read
            return oflags | (1 << 5); // O_RDWR
        }
        return oflags | (1 << 4); // O_WRONLY
    }
    oflags
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_openZ_iiiiiijjii(
    ctx: *const *mut VmCtx,
    fd: u32,
    dirflags: u32,
    path: u32,
    path_len: u32,
    oflags: u32,
    fs_rights_base: u64,
    _fs_rights_inheriting: u64,
    fdflags: u32,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    // adjust oflags by adding O_WRONLY & O_RDWR as bits 4 and 5
    // after wasi-libc put them in fs_rights_base
    let new_flags = adjust_oflags(oflags, fs_rights_base);
    let r = wasi_path_open(
        ctx_ref,
        fd,
        dirflags,
        path,
        path_len,
        new_flags,
        fdflags as i32,
    );
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("path_open", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_readlinkZ_iiiiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    path: u32,
    path_len: u32,
    buf: u32,
    buf_len: u32,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_readlink(ctx_ref, fd, path, path_len, buf, buf_len);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("path_readlink", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
// Pass through path_len
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_remove_directoryZ_iiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    path: u32,
    path_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_remove_directory(ctx_ref, fd, path, path_len);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("path_remove_directory", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_renameZ_iiiiiii(
    ctx: *const *mut VmCtx,
    old_fd: u32,
    old_path: u32,
    old_path_len: u32,
    new_fd: u32,
    new_path: u32,
    new_path_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_rename(
        ctx_ref,
        old_fd,
        old_path,
        old_path_len,
        new_fd,
        new_path,
        new_path_len,
    );
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("path_rename", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_symlinkZ_iiiiii(
    ctx: *const *mut VmCtx,
    old_path: u32,
    old_path_len: u32,
    fd: u32,
    path: u32,
    path_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_symlink(ctx_ref, old_path, old_path_len, fd, path, path_len);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("path_symlink", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_unlink_fileZ_iiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    path: u32,
    path_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_unlink_file(ctx_ref, fd, path, path_len);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("path_unlink_file", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_poll_oneoffZ_iiiii(
    ctx: *const *mut VmCtx,
    in_ptr: u32,
    out_ptr: u32,
    nsubscriptions: u32,
    retptr: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_poll_oneoff(ctx_ref, in_ptr, out_ptr, nsubscriptions);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r);
    let end = stop_timer();
    push_hostcall_result("poll_oneoff", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_proc_raiseZ_ii(
    ctx: *const *mut VmCtx,
    signal: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_proc_raise(ctx_ref, signal);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("proc_raise", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_random_getZ_iii(
    ctx: *const *mut VmCtx,
    buf: u32,
    buf_len: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_random_get(ctx_ref, buf, buf_len);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("random_get", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sched_yieldZ_iv(ctx: *const *mut VmCtx) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sched_yield(ctx_ref);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("sched_yield", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sock_recvZ_iiiiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    ri_data: u32,
    ri_data_count: u32,
    ri_flags: u32,
    out0: u32,
    out1: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_recv(ctx_ref, fd, ri_data, ri_data_count, ri_flags);
    let retval = wasm2c_marshal_and_writeback_u32_pair(ctx_ref, out0 as usize, out1 as usize, r);
    let end = stop_timer();
    push_hostcall_result("sock_recv", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sock_sendZ_iiiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    si_data: u32,
    si_data_count: u32,
    si_flags: u32,
    out: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_send(ctx_ref, fd, si_data, si_data_count, si_flags);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r);
    let end = stop_timer();
    push_hostcall_result("sock_send", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sock_shutdownZ_iii(
    ctx: *const *mut VmCtx,
    fd: u32,
    how: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_shutdown(ctx_ref, fd, how);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("sock_shutdown", start, end);
    retval
}

/*
 New Calls
*/

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_socketZ_iiiii(
    ctx: *const *mut VmCtx,
    domain: u32,
    ty: u32,
    protocol: u32,
    retptr: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_socket(ctx_ref, domain, ty, protocol);
    let retval = wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r);
    let end = stop_timer();
    push_hostcall_result("socket", start, end);
    retval
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sock_connectZ_iiii(
    ctx: *const *mut VmCtx,
    sockfd: u32,
    addr: u32,
    addrlen: u32,
) -> u32 {
    let start = start_timer();
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_connect(ctx_ref, sockfd, addr, addrlen);
    let retval = wasm2c_marshal(r);
    let end = stop_timer();
    push_hostcall_result("sock_connect", start, end);
    retval
}
