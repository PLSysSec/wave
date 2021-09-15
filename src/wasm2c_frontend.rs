use crate::trace::Trace;
use crate::types::*;
use crate::wrappers::*;
use trace::trace;
use RuntimeError::*;

trace::init_depth_var!();

//TODO: figure out how to remove the dummy traces

/// Used for FFI. (wasm2c frontend)
/// Initialize a vmctx with a memory that points to memptr
fn ctx_from_memptr(memptr: *mut u8, memsize: isize, homedir: String) -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    //let mem = vec![0; memlen];
    let mem = unsafe { Vec::from_raw_parts(memptr, memlen, memlen) };
    let mut fdmap = FdMap::new();
    fdmap.init_std_fds();
    let arg_buffer = vec![b'\0'];
    let argc = 0;
    let env_buffer = vec![b'\0'];
    let envc = 0;

    VmCtx {
        mem,
        memlen,
        fdmap,
        homedir,
        errno: Success,
        arg_buffer,
        argc,
        env_buffer,
        envc,
    }
}

/// To get wasm2c ffi working, we need to pass a VmCtx pointer back and forth
/// from C to Rust and back again.
/// The actual pointer that wasm2c gives us has a second layer of indrection
/// so we deref it twice to get the vmctx, then return a reference to that VmCtx
fn ptr_to_ref(ctx: *const *mut VmCtx) -> &'static mut VmCtx {
    if ctx.is_null() {
        panic!("null ctx")
    }
    unsafe { &mut **ctx }
}

fn wasm2c_marshal<T>(result: RuntimeResult<T>) -> u32{
    match result {
        Ok(r) => 0,
        Err(err) => err.into(),
    }
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
    ctx: *const *mut VmCtx,
    argv: u32,
    argv_buf: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_args_get(ctx_ref, argv, argv_buf);
    return wasm2c_marshal(r);
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_args_sizes_getZ_iii(
    ctx: *const *mut VmCtx,
    pargc: u32,
    pargv_buf_size: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    if let Ok((argc, argv_buf)) = wasi_args_sizes_get(ctx_ref){
        ctx_ref.write_u32(pargc as usize, argc as u32); // writeback argc
        ctx_ref.write_u32(pargv_buf_size as usize, argv_buf as u32); // writeback argv
    }
    return 0;
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_proc_exitZ_vi(ctx: *const *mut VmCtx, x: u32) {
    let ctx_ref = ptr_to_ref(ctx);
    wasi_proc_exit(ctx_ref, x);
}


#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_sizes_getZ_iii(
    ctx: *const *mut VmCtx,
    pcount: u32,
    pbuf_size: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    if let Ok((envc, env_buf)) = wasi_environ_sizes_get(ctx_ref){
        ctx_ref.write_u32(pcount as usize, envc as u32); // writeback envc
        ctx_ref.write_u32(pbuf_size as usize, env_buf as u32); // writeback environ_buf
    }
    return 0;
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_getZ_iii(
    ctx: *const *mut VmCtx,
    __environ: u32,
    environ_buf: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_environ_get(ctx_ref, __environ, environ_buf);
    return wasm2c_marshal(r);
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_prestat_getZ_iii(
    ctx: *const *mut VmCtx,
    fd: u32,
    prestat: u32,
) -> u32 {
    // Wasm2c implementation
    // TODO: Should probably replace with a real implementation based
    // on wasi-common's
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_prestat_get(ctx_ref, fd);
    return wasm2c_marshal(r);
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_writeZ_iiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    iov: u32,
    iovcnt: u32,
    pnum: u32,
) -> u32 {
    // TODO: write back to pnum
    let ctx_ref = ptr_to_ref(ctx);
    let mut dummy_trace = Trace::new();

    match wasi_fd_write(ctx_ref, fd, iov, iovcnt, &mut dummy_trace) {
        Ok(result) => {
            ctx_ref.write_u32(pnum as usize, result); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}



#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_readZ_iiiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    iov: u32,
    iovcnt: u32,
    pnum: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let mut dummy_trace = Trace::new();
    match wasi_fd_read(ctx_ref, fd, iov, iovcnt, &mut dummy_trace) {
        Ok(result) => {
            ctx_ref.write_u32(pnum as usize, result); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_closeZ_ii(ctx: *const *mut VmCtx, fd: u32) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_close(ctx_ref, fd);
    return wasm2c_marshal(r);
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_seekZ_iijii(
    ctx: *const *mut VmCtx,
    fd: u32,
    offset: u64,
    whence: u32,
    new_offset: u32, // output
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    if let Some(converted_whence) = Whence::from_u32(whence){
        match wasi_fd_seek(ctx_ref, fd, offset as i64, converted_whence) {
            Ok(result) => {
                ctx_ref.write_u32(new_offset as usize, result as u32); // writeback result
                0
            }
            Err(err) => err.into(),
        }
    }
    else{
        return 28; // WASI_INVAL_ERROR
    }
}

#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_time_getZ_iiji(
    ctx: *const *mut VmCtx,
    clock_id: u32,
    // max_lag: u64,
    out: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    if let Some(converted_clock_id) = ClockId::from_u32(clock_id){
        match wasi_clock_time_get(ctx_ref, converted_clock_id) {
            Ok(result) => {
                ctx_ref.write_u64(out as usize, result.nsec()); // writeback result
                0
            }
            Err(err) => err.into(),
        }
    }
    else{
        return 28; //WASI_INVAL_ERROR
    }
}
#[no_mangle]
#[trace]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_res_getZ_iii(
    ctx: *const *mut VmCtx,
    clock_id: u32,
    out: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    if let Some(converted_clock_id) = ClockId::from_u32(clock_id){
        match wasi_clock_res_get(ctx_ref, converted_clock_id ) {
            Ok(result) => {
                ctx_ref.write_u64(out as usize, result.nsec()); // writeback result
                0
            }
            Err(err) => err.into(),
        }
    }
    else{
        return 28; //WASI_INVAL_ERROR
    }
    
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
