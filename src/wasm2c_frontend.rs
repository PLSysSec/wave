use crate::types::*;
use crate::wrappers::*;
use trace::trace;
use RuntimeError::*;

trace::init_depth_var!();

/// Used for FFI. (wasm2c frontend)
/// Initialize a vmctx with a memory that points to memptr
fn ctx_from_memptr(memptr: *mut u8, memsize: isize, homedir: String) -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    //let mem = vec![0; memlen];
    let mem = unsafe { Vec::from_raw_parts(memptr, memlen, memlen) };
    let fdmap = FdMap::new();
    VmCtx {
        mem,
        memlen,
        fdmap,
        homedir,
        errno: Success,
    }
}

fn ptr_to_ref(ctx: *mut VmCtx) -> &'static mut VmCtx {
    if ctx.is_null() {
        panic!("null ctx")
    }
    unsafe { &mut *ctx }
}

#[no_mangle]
#[trace]
pub extern "C" fn veriwasi_init(memptr: *mut u8, memsize: isize) -> *mut VmCtx {
    let ctx = ctx_from_memptr(memptr, memsize, "/".to_string());
    let result = Box::into_raw(Box::new(ctx));
    result
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_args_getZ_iii(
    ctx: *mut VmCtx,
    argv: u32,
    argv_buf: u32,
) -> u32 {
    unimplemented!()
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_args_sizes_getZ_iii(
    ctx: *mut VmCtx,
    pargc: u32,
    pargv_buf_size: u32,
) -> u32 {
    unimplemented!()
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_proc_exitZ_vi(ctx: *mut VmCtx, x: u32) {
    unimplemented!()
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_sizes_getZ_iii(
    ctx: *mut VmCtx,
    pcount: u32,
    pbuf_size: u32,
) -> u32 {
    unimplemented!()
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_getZ_iii(
    ctx: *mut VmCtx,
    __environ: u32,
    environ_buf: u32,
) -> u32 {
    unimplemented!()
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_prestat_getZ_iii(
    ctx: *mut VmCtx,
    fd: u32,
    prestat: u32,
) -> u32 {
    unimplemented!()
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_writeZ_iiiii(
    ctx: *mut VmCtx,
    fd: u32,
    iov: u32,
    iovcnt: u32,
    pnum: u32,
) -> u32 {
    // TODO: write back to pnum
    let ctx_ref = ptr_to_ref(ctx);
    let result = wasi_fd_write(ctx_ref, fd, iov, iovcnt);
    result
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_readZ_iiiii(
    ctx: *mut VmCtx,
    fd: u32,
    iov: u32,
    iovcnt: u32,
    pnum: u32,
) -> u32 {
    // TODO: writeback to pnum
    let ctx_ref = ptr_to_ref(ctx);
    let result = wasi_fd_read(ctx_ref, fd, iov, iovcnt);
    result
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_closeZ_ii(ctx: *mut VmCtx, fd: u32) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    wasi_fd_close(ctx_ref, fd)
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_seekZ_iijii(
    ctx: *mut VmCtx,
    fd: u32,
    offset: u64,
    whence: u32,
    new_offset: u32,
) -> u32 {
    unimplemented!()
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_seekZ_iiiiii(
    ctx: *mut VmCtx,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
) -> u32 {
    unimplemented!()
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_time_getZ_iiji(
    ctx: *mut VmCtx,
    clock_id: u32,
    max_lag: u64,
    out: u32,
) -> u32 {
    unimplemented!()
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_res_getZ_iii(
    ctx: *mut VmCtx,
    clock_id: u32,
    out: u32,
) -> u32 {
    unimplemented!()
}

// void wasm_rt_sys_init() {
// void wasm_rt_init_wasi(wasm_sandbox_wasi_data* wasi_data) {
// void wasm_rt_cleanup_wasi(wasm_sandbox_wasi_data* wasi_data) {

/*
Wasi API that is not currently supported by wasm2c
*/

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_adviseZ_iijji(a: u32, b: u64, c: u64, d: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_allocateZ_iijj(a: u32, b: u64, c: u64) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_datasyncZ_ii(a: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_fdstat_getZ_iii(a: u32, b: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_fdstat_set_flagsZ_iii(a: u32, b: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_fdstat_set_rightsZ_iijj(a: u32, b: u64, c: u64) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_filestat_getZ_iii(a: u32, b: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_filestat_set_sizeZ_iij(a: u32, b: u64) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_filestat_set_timesZ_iijji(a: u32, b: u64, c: u64, d: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_preadZ_iiiiji(a: u32, b: u32, c: u32, d: u64, e: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_prestat_dir_nameZ_iiii(a: u32, b: u32, c: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_pwriteZ_iiiiji(a: u32, b: u32, c: u32, d: u64, e: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_readdirZ_iiiiji(a: u32, b: u32, c: u32, d: u64, e: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_renumberZ_iii(a: u32, b: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_syncZ_ii(a: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_tellZ_iii(a: u32, b: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_create_directoryZ_iiii(a: u32, b: u32, c: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_filestat_getZ_iiiiii(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_filestat_set_timesZ_iiiiijji(a: u32, b: u32, c: u32, d: u32, e: u64, f: u64, g: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_linkZ_iiiiiiii(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32) -> u32 {
//     unimplemented!()
// }

/*
fd: fd, dirflags: lookupflags, path: string, oflags: oflags, fs_rights_base: rights, fs_rights_inheriting: rights, fdflags: fdflags
*/

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_openZ_iiiiiijjii(ctx: &mut VmCtx, a: u32, b: u32, c: u32, d: u32, e: u32, f: u64, g: u64, h: u32, i: u32) -> u32 {
//     wasi_path_open(ctx, a, b, c, d, e, f, g, h, i)
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_readlinkZ_iiiiiii(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_remove_directoryZ_iiii(a: u32, b: u32, c: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_renameZ_iiiiiii(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32{
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_symlinkZ_iiiiii(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_path_unlink_fileZ_iiii(a: u32, b: u32, c: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_poll_oneoffZ_iiiii(a: u32, b: u32, c: u32, d: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_proc_raiseZ_ii(a: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_random_getZ_iii(a: u32, b: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_sched_yieldZ_i() -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_sock_recvZ_iiiiiii(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_sock_sendZ_iiiiii(a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
//     unimplemented!()
// }

// #[no_mangle]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_sock_shutdownZ_iii(a: u32, b: u32) -> u32 {
//     unimplemented!()
// }
