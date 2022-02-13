use crate::tcb::ffi::*;
use crate::types::*;
use crate::wasm2c_frontend::*;
use crate::wrappers::*;
use crate::writeback::*;
use std::panic;
use trace::trace;
use RuntimeError::*;

trace::init_depth_var!();

// Helpers for building initial VmCtx
// Intended to make lucet integration cleaner

pub fn redirect_stdout(ctx: &mut VmCtx, new_stdout: i32) -> () {
    ctx.fdmap.m[1] = Ok((new_stdout as usize).into())
}

pub fn add_arg(ctx: &mut VmCtx, arg: String) -> () {
    if ctx.argc != 0 {
        ctx.arg_buffer.push(0);
    }
    ctx.argc += 1;
    ctx.arg_buffer.extend(arg.into_bytes());
}

pub fn add_env_var(ctx: &mut VmCtx, env_var: String) -> () {
    if ctx.envc != 0 {
        ctx.env_buffer.push(0);
    }
    ctx.envc += 1;
    ctx.env_buffer.extend(env_var.into_bytes());
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_args_get_wave(
    ctx: *const *mut VmCtx,
    argv: i32,
    argv_buf: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_args_getZ_iii(ctx, argv as u32, argv_buf as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_args_sizes_get_wave(
    ctx: *const *mut VmCtx,
    pargc: i32,
    pargv_buf_size: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_args_sizes_getZ_iii(ctx, pargc as u32, pargv_buf_size as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_proc_exit_wave(ctx: *const *mut VmCtx, x: i32) {
    std::panic::panic_any(3);
    //panic!(x);
    //panic::resume_unwind(Box::new("this is an exit!"))
    //Z_wasi_snapshot_preview1Z_proc_exitZ_vi(ctx, x as u32)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_environ_sizes_get_wave(
    ctx: *const *mut VmCtx,
    pcount: i32,
    pbuf_size: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_environ_sizes_getZ_iii(ctx, pcount as u32, pbuf_size as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_environ_get_wave(
    ctx: *const *mut VmCtx,
    __environ: i32,
    environ_buf: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_environ_getZ_iii(ctx, __environ as u32, environ_buf as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_prestat_get_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    prestat: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_prestat_getZ_iii(ctx, fd as u32, prestat as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_write_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    iov: i32,
    iovcnt: i32,
    pnum: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_writeZ_iiiii(
        ctx,
        fd as u32,
        iov as u32,
        iovcnt as u32,
        pnum as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_read_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    iov: i32,
    iovcnt: i32,
    pnum: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_readZ_iiiii(ctx, fd as u32, iov as u32, iovcnt as u32, pnum as u32)
        as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_close_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_closeZ_ii(ctx, fd as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_seek_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    offset: i64,
    whence: i32,
    new_offset: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_seekZ_iijii(
        ctx,
        fd as u32,
        offset as u64,
        whence as u32,
        new_offset as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_clock_time_get_wave(
    ctx: *const *mut VmCtx,
    clock_id: i32,
    precision: i64,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_clock_time_getZ_iiji(
        ctx,
        clock_id as u32,
        precision as u64,
        out as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_clock_res_get_wave(
    ctx: *const *mut VmCtx,
    clock_id: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_clock_res_getZ_iii(ctx, clock_id as u32, out as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_advise_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
    offset: i64,
    len: i64,
    advice: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_adviseZ_iijji(
        ctx,
        v_fd as u32,
        offset as u64,
        len as u64,
        advice as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_allocate_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
    offset: i64,
    len: i64,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_allocateZ_iijj(ctx, v_fd as u32, offset as u64, len as u64) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_datasync_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_datasyncZ_ii(ctx, v_fd as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_fdstat_get_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_fdstat_getZ_iii(ctx, v_fd as u32, out as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_fdstat_set_flags_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
    flags: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_fdstat_set_flagsZ_iii(ctx, v_fd as u32, flags as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_fdstat_set_rights_wave(
    ctx: *const *mut VmCtx,
    a: i32,
    b: i64,
    c: i64,
) -> i32 {
    _Z_wasi_snapshot_preview1Z_fd_fdstat_set_rightsZ_iijj(ctx, a as u32, b as u64, c as u64) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_filestat_get_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_filestat_getZ_iii(ctx, v_fd as u32, out as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_filestat_set_size_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
    size: i64,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_filestat_set_sizeZ_iij(ctx, v_fd as u32, size as u64) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_filestat_set_times_wave(
    ctx: *const *mut VmCtx,
    v_fd: i32,
    atim: i64,
    mtim: i64,
    fst_flags: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_filestat_set_timesZ_iijji(
        ctx,
        v_fd as u32,
        atim as u64,
        mtim as u64,
        fst_flags as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_pread_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    iovs: i32,
    iov_len: i32,
    offset: i64,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_preadZ_iiiiji(
        ctx,
        fd as u32,
        iovs as u32,
        iov_len as u32,
        offset as u64,
        out as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_prestat_dir_name_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    path: i32,
    path_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_prestat_dir_nameZ_iiii(
        ctx,
        fd as u32,
        path as u32,
        path_len as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_pwrite_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    iovs: i32,
    iov_len: i32,
    offset: i64,
    retptr: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_pwriteZ_iiiiji(
        ctx,
        fd as u32,
        iovs as u32,
        iov_len as u32,
        offset as u64,
        retptr as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_readdir_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    buf: i32,
    buf_len: i32,
    cookie: i64,
    retptr: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_readdirZ_iiiiji(
        ctx,
        fd as u32,
        buf as u32,
        buf_len as u32,
        cookie as u64,
        retptr as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_renumber_wave(
    ctx: *const *mut VmCtx,
    from: i32,
    to: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_renumberZ_iii(ctx, from as u32, to as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_sync_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_syncZ_ii(ctx, fd as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_fd_tell_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_fd_tellZ_iii(ctx, fd as u32, out as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_create_directory_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    pathname: i32,
    path_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_create_directoryZ_iiii(
        ctx,
        fd as u32,
        pathname as u32,
        path_len as u32,
    ) as i32
}

// wasm2c and wasi-libc disagree about 4 vs 5 arguments
#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_filestat_get_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    flags: i32,
    path: i32,
    path_len: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_filestat_getZ_iiiiii(
        ctx,
        fd as u32,
        flags as u32,
        path as u32,
        path_len as u32,
        out as u32,
    ) as i32
}

// wasi-libc and wasm2c disagree about whether this arg should exist
#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_filestat_set_times_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    flags: i32,
    path: i32,
    path_len: i32,
    atim: i64,
    mtim: i64,
    fst_flags: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_filestat_set_timesZ_iiiiijji(
        ctx,
        fd as u32,
        flags as u32,
        path as u32,
        path_len as u32,
        atim as u64,
        mtim as u64,
        fst_flags as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_link_wave(
    ctx: *const *mut VmCtx,
    old_fd: i32,
    old_flags: i32,
    old_path: i32,
    old_path_len: i32,
    new_fd: i32,
    new_path: i32,
    new_path_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_linkZ_iiiiiiii(
        ctx,
        old_fd as u32,
        old_flags as u32,
        old_path as u32,
        old_path_len as u32,
        new_fd as u32,
        new_path as u32,
        new_path_len as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_open_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    dirflags: i32,
    path: i32,
    path_len: i32,
    oflags: i32,
    fs_rights_base: i64,
    _fs_rights_inheriting: i64,
    fdflags: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_openZ_iiiiiijjii(
        ctx,
        fd as u32,
        dirflags as u32,
        path as u32,
        path_len as u32,
        oflags as u32,
        fs_rights_base as u64,
        _fs_rights_inheriting as u64,
        fdflags as u32,
        out as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_readlink_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    path: i32,
    path_len: i32,
    buf: i32,
    buf_len: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_readlinkZ_iiiiiii(
        ctx,
        fd as u32,
        path as u32,
        path_len as u32,
        buf as u32,
        buf_len as u32,
        out as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_remove_directory_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    path: i32,
    path_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_remove_directoryZ_iiii(
        ctx,
        fd as u32,
        path as u32,
        path_len as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_rename_wave(
    ctx: *const *mut VmCtx,
    old_fd: i32,
    old_path: i32,
    old_path_len: i32,
    new_fd: i32,
    new_path: i32,
    new_path_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_renameZ_iiiiiii(
        ctx,
        old_fd as u32,
        old_path as u32,
        old_path_len as u32,
        new_fd as u32,
        new_path as u32,
        new_path_len as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_symlink_wave(
    ctx: *const *mut VmCtx,
    old_path: i32,
    old_path_len: i32,
    fd: i32,
    path: i32,
    path_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_symlinkZ_iiiiii(
        ctx,
        old_path as u32,
        old_path_len as u32,
        fd as u32,
        path as u32,
        path_len as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_path_unlink_file_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    path: i32,
    path_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_path_unlink_fileZ_iiii(ctx, fd as u32, path as u32, path_len as u32)
        as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_poll_oneoff_wave(
    ctx: *const *mut VmCtx,
    in_ptr: i32,
    out_ptr: i32,
    nsubscriptions: i32,
    retptr: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_poll_oneoffZ_iiiii(
        ctx,
        in_ptr as u32,
        out_ptr as u32,
        nsubscriptions as u32,
        retptr as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_proc_raise_wave(
    ctx: *const *mut VmCtx,
    signal: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_proc_raiseZ_ii(ctx, signal as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_random_get_wave(
    ctx: *const *mut VmCtx,
    buf: i32,
    buf_len: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_random_getZ_iii(ctx, buf as u32, buf_len as u32) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_sched_yield_wave(ctx: *const *mut VmCtx) -> i32 {
    Z_wasi_snapshot_preview1Z_sched_yieldZ_iv(ctx) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_sock_recv_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    ri_data: i32,
    ri_data_count: i32,
    ri_flags: i32,
    out0: i32,
    out1: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_sock_recvZ_iiiiiii(
        ctx,
        fd as u32,
        ri_data as u32,
        ri_data_count as u32,
        ri_flags as u32,
        out0 as u32,
        out1 as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_sock_send_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    si_data: i32,
    si_data_count: i32,
    si_flags: i32,
    out: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_sock_sendZ_iiiiii(
        ctx,
        fd as u32,
        si_data as u32,
        si_data_count as u32,
        si_flags as u32,
        out as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_sock_shutdown_wave(
    ctx: *const *mut VmCtx,
    fd: i32,
    how: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_sock_shutdownZ_iii(ctx, fd as u32, how as u32) as i32
}

/*
 New Calls
*/

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_socket_wave(
    ctx: *const *mut VmCtx,
    domain: i32,
    ty: i32,
    protocol: i32,
    retptr: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_socketZ_iiiii(
        ctx,
        domain as u32,
        ty as u32,
        protocol as u32,
        retptr as u32,
    ) as i32
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn hostcall_wasi_snapshot_preview1_sock_connect_wave(
    ctx: *const *mut VmCtx,
    sockfd: i32,
    addr: i32,
    addrlen: i32,
) -> i32 {
    Z_wasi_snapshot_preview1Z_sock_connectZ_iiii(ctx, sockfd as u32, addr as u32, addrlen as u32)
        as i32
}
