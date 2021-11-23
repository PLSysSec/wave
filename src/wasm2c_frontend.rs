use crate::types::*;
use crate::wrappers::*;
use libc::{c_char, strlen};
use std::ffi::CStr;
use std::os::unix::io::AsRawFd;
use std::time::Instant;
use trace::trace;
use RuntimeError::*;
// use log::{debug, error, log_enabled, info, Level};
// use log;
// use env_logger;

trace::init_depth_var!();

// wasi_call_void!(ctx, func_name )
// needs to accept # of args?
// also needs to rename function
// macro_rules! wasi_call_void {
//     ($ctx:ident, $func_name:ident) => {
//         if cfg!(feature = not("verify")) {
//             let ctx_ref = ptr_to_ref($ctx);
//             let r = $func_name(ctx_ref, argv, argv_buf);
//             wasm2c_marshal(r)
//         }
//     }
// }

//TODO: figure out how to remove the dummy trace(logging)s

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
) -> VmCtx {
    //env_logger::init();
    let builder = env_logger::Builder::from_default_env()
        .format_timestamp_nanos()
        .init();

    println!(
        "homedir = {:?} args = {:?} env = {:?} log_path = {:?}",
        homedir, args, env, log_path
    );
    let memlen = LINEAR_MEM_SIZE;
    //let mem = vec![0; memlen];
    let mem = unsafe { Vec::from_raw_parts(memptr, memlen, memlen) };
    let mut fdmap = FdMap::new();
    fdmap.init_std_fds();
    // let homedir_fd = std::fs::File::open(&homedir).unwrap().as_raw_fd();
    let log_path = unsafe { CStr::from_ptr(log_path).to_str().unwrap().to_owned() }.clone();

    let homedir = unsafe { CStr::from_ptr(homedir).to_str().unwrap() }.clone();
    let homedir_file = std::fs::File::open(homedir).unwrap();
    let homedir_fd = homedir_file.as_raw_fd();
    if homedir_fd > 0 {
        fdmap.create((homedir_fd as usize).into());
    }
    // Need to forget file to make sure it does not get auto-closed
    // when it gets out of scope
    std::mem::forget(homedir_file);

    let arg_len = unsafe { strlen(args as *const i8) };
    let mut arg_buffer = unsafe { Vec::from_raw_parts(args, arg_len, arg_len) }.clone();
    // replace all space with null.
    // This makes it easy to return the arg_buffer later
    for i in 0..arg_len {
        if arg_buffer[i] == b' ' {
            arg_buffer[i] = b'\0';
        }
    }

    let env_len = unsafe { strlen(env as *const i8) };
    let mut env_buffer = unsafe { Vec::from_raw_parts(env, env_len, env_len) }.clone();
    // replace all space with null.
    // This makes it easy to return the env_buffer later
    for i in 0..env_len {
        if env_buffer[i] == b' ' {
            env_buffer[i] = b'\0';
        }
    }

    VmCtx {
        mem,
        memlen,
        fdmap,
        homedir: homedir.to_owned(),
        errno: Success,
        arg_buffer,
        argc,
        env_buffer,
        envc,
        log_path,
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

fn wasm2c_marshal<T>(result: RuntimeResult<T>) -> u32 {
    match result {
        Ok(r) => 0,
        Err(err) => err.into(),
    }
}

fn wasm2c_marshal_and_writeback_u32(
    ctx_ref: &mut VmCtx,
    addr: usize,
    result: RuntimeResult<u32>,
) -> u32 {
    log::info!("wasm2c_marshal_and_writeback_u32: {:?}", result);
    match result {
        Ok(r) => {
            ctx_ref.write_u32(addr, r); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

fn wasm2c_marshal_and_writeback_prestat(
    ctx_ref: &mut VmCtx,
    addr: usize,
    result: RuntimeResult<u32>,
) -> u32 {
    log::info!("wasm2c_marshal_and_writeback_prestat: {:?}", result);
    match result {
        Ok(r) => {
            ctx_ref.write_u32(addr, 0);
            ctx_ref.write_u64(addr + 4, r as u64); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

fn wasm2c_marshal_and_writeback_u64(
    ctx_ref: &mut VmCtx,
    addr: usize,
    result: RuntimeResult<u64>,
) -> u32 {
    log::info!("wasm2c_marshal_and_writeback_u64: {:?}", result);
    match result {
        Ok(r) => {
            ctx_ref.write_u64(addr, r); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

fn wasm2c_marshal_and_writeback_timestamp(
    ctx_ref: &mut VmCtx,
    addr: usize,
    result: RuntimeResult<Timestamp>,
) -> u32 {
    log::info!("wasm2c_marshal_and_writeback_timestamp: {:?}", result);
    match result {
        Ok(r) => {
            ctx_ref.write_u64(addr, r.nsec()); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

fn wasm2c_marshal_and_writeback_fdstat(
    ctx_ref: &mut VmCtx,
    addr: usize,
    result: RuntimeResult<FdStat>,
) -> u32 {
    log::info!("wasm2c_marshal_and_writeback_fdstat: {:?}", result);
    match result {
        Ok(r) => {
            ctx_ref.write_u16(addr, r.fs_filetype.to_wasi() as u16);
            ctx_ref.write_u16(addr + 2, r.fs_flags.to_posix() as u16);
            ctx_ref.write_u64(addr + 8, r.fs_rights_base);
            ctx_ref.write_u64(addr + 16, r.fs_rights_inheriting);
            0
        }
        Err(err) => err.into(),
    }
}

fn wasm2c_marshal_and_writeback_filestat(
    ctx_ref: &mut VmCtx,
    addr: usize,
    result: RuntimeResult<FileStat>,
) -> u32 {
    log::info!("wasm2c_marshal_and_writeback_filestat: {:?}", result);
    match result {
        Ok(r) => {
            ctx_ref.write_u64(addr, r.dev);
            ctx_ref.write_u64(addr + 8, r.ino);
            ctx_ref.write_u64(addr + 16, r.filetype as u64);
            ctx_ref.write_u64(addr + 24, r.nlink);
            ctx_ref.write_u64(addr + 32, r.size);
            ctx_ref.write_u64(addr + 40, r.atim.nsec());
            ctx_ref.write_u64(addr + 48, r.mtim.nsec());
            ctx_ref.write_u64(addr + 56, r.ctim.nsec());
            0
        }
        Err(err) => err.into(),
    }
}

fn wasm2c_marshal_and_writeback_u32_pair(
    ctx_ref: &mut VmCtx,
    addr0: usize,
    addr1: usize,
    result: RuntimeResult<(u32, u32)>,
) -> u32 {
    log::info!("wasm2c_marshal_and_writeback_u32_pair: {:?}", result);
    match result {
        Ok((v0, v1)) => {
            ctx_ref.write_u32(addr0, v0); // writeback envc
            ctx_ref.write_u32(addr1, v1); // writeback environ_buf
            0
        }
        Err(err) => err.into(),
    }
}

// TODO: let us pass through what the homedir is from the cmdline
#[no_mangle]
#[trace(logging)]
pub extern "C" fn veriwasi_init(
    memptr: *mut u8,
    memsize: isize,
    homedir: *const c_char,
    args: *mut u8,
    argc: usize,
    env: *mut u8,
    envc: usize,
    log_path: *mut c_char,
) -> *mut VmCtx {
    let ctx = ctx_from_memptr(memptr, memsize, homedir, args, argc, env, envc, log_path);
    let result = Box::into_raw(Box::new(ctx));
    result
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_args_getZ_iii(
    ctx: *const *mut VmCtx,
    argv: u32,
    argv_buf: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_args_get(ctx_ref, argv, argv_buf);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_args_sizes_getZ_iii(
    ctx: *const *mut VmCtx,
    pargc: u32,
    pargv_buf_size: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_args_sizes_get(ctx_ref);
    wasm2c_marshal_and_writeback_u32_pair(ctx_ref, pargc as usize, pargv_buf_size as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_proc_exitZ_vi(ctx: *const *mut VmCtx, x: u32) {
    std::process::exit(x as i32);
    // let ctx_ref = ptr_to_ref(ctx);
    // wasi_proc_exit(ctx_ref, x);
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_sizes_getZ_iii(
    ctx: *const *mut VmCtx,
    pcount: u32,
    pbuf_size: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_environ_sizes_get(ctx_ref);
    wasm2c_marshal_and_writeback_u32_pair(ctx_ref, pcount as usize, pbuf_size as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_environ_getZ_iii(
    ctx: *const *mut VmCtx,
    __environ: u32,
    environ_buf: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_environ_get(ctx_ref, __environ, environ_buf);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_prestat_getZ_iii(
    ctx: *const *mut VmCtx,
    fd: u32,
    prestat: u32,
) -> u32 {
    // Wasm2c implementation
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_prestat_get(ctx_ref, fd);
    wasm2c_marshal_and_writeback_prestat(ctx_ref, prestat as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_write(ctx_ref, fd, iov, iovcnt);
    wasm2c_marshal_and_writeback_u32(ctx_ref, pnum as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_read(ctx_ref, fd, iov, iovcnt);
    wasm2c_marshal_and_writeback_u32(ctx_ref, pnum as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_closeZ_ii(ctx: *const *mut VmCtx, fd: u32) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_close(ctx_ref, fd);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_seek(ctx_ref, fd, offset as i64, whence);
    wasm2c_marshal_and_writeback_u32(ctx_ref, new_offset as usize, r)
}

// TODO: what is the purpose of precision here?
// TODO: this might be off by a factor of 1000
#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_time_getZ_iiji(
    ctx: *const *mut VmCtx,
    clock_id: u32,
    precision: u64,
    out: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_clock_time_get(ctx_ref, clock_id);
    wasm2c_marshal_and_writeback_timestamp(ctx_ref, out as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_clock_res_getZ_iii(
    ctx: *const *mut VmCtx,
    clock_id: u32,
    out: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_clock_res_get(ctx_ref, clock_id);
    wasm2c_marshal_and_writeback_timestamp(ctx_ref, out as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_advise(ctx_ref, v_fd, offset, len, advice);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_allocateZ_iijj(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    offset: u64,
    len: u64,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_allocate(ctx_ref, v_fd, offset, len);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_datasyncZ_ii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_datasync(ctx_ref, v_fd);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_fdstat_getZ_iii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    out: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_fdstat_get(ctx_ref, v_fd);
    wasm2c_marshal_and_writeback_fdstat(ctx_ref, out as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_fdstat_set_flagsZ_iii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    flags: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_fdstat_set_flags(ctx_ref, v_fd, flags);
    wasm2c_marshal(r)
}

// Not supporting this because rights are getting removed
// #[no_mangle]
// #[trace(logging)]
// pub extern "C" fn _Z_wasi_snapshot_preview1Z_fd_fdstat_set_rightsZ_iijj(ctx: *const *mut VmCtx, a: u32, b: u64, c: u64) -> u32 {
//     unimplemented!()
// }

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_filestat_getZ_iii(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    out: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_filestat_get(ctx_ref, v_fd);
    wasm2c_marshal_and_writeback_filestat(ctx_ref, out as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_filestat_set_sizeZ_iij(
    ctx: *const *mut VmCtx,
    v_fd: u32,
    size: u64,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_filestat_set_size(ctx_ref, v_fd, size as i64);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_filestat_set_times(ctx_ref, v_fd, atim, mtim, fst_flags);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_pread(ctx_ref, fd, iovs, iov_len, offset);
    wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_prestat_dir_nameZ_iiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    path: u32,
    path_len: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_prestat_dirname(ctx_ref, fd, path, path_len);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_pwrite(ctx_ref, fd, iovs, iov_len, offset);
    wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_readdir(ctx_ref, fd, buf, buf_len as usize, cookie);
    wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_renumberZ_iii(
    ctx: *const *mut VmCtx,
    from: u32,
    to: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_renumber(ctx_ref, from, to);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_syncZ_ii(ctx: *const *mut VmCtx, fd: u32) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_sync(ctx_ref, fd);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_fd_tellZ_iii(
    ctx: *const *mut VmCtx,
    fd: u32,
    out: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_fd_tell(ctx_ref, fd);
    wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_create_directoryZ_iiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    pathname: u32,
    path_len: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_create_directory(ctx_ref, fd, pathname, path_len);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_filestat_get(ctx_ref, fd, flags, path, path_len);
    wasm2c_marshal_and_writeback_filestat(ctx_ref, out as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_filestat_set_times(
        ctx_ref,
        fd,
        flags,
        path,
        path_len,
        atim,
        mtim,
        FstFlags::new(fst_flags as u16),
    );
    wasm2c_marshal(r)
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
    wasm2c_marshal(r)
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
    return oflags;
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
    let ctx_ref = ptr_to_ref(ctx);
    // adjust oflags by adding O_WRONLY & O_RDWR as bits 4 and 5
    // after wasi-libc put them in fs_rights_base
    // let new_flags =
    // if fs_rights_base & (1 << 6) != 0 { // can_write
    //     if fs_rights_base & (1 << 1) != 0 { // can read
    //         oflags | (1 << 5) // O_RDWR
    //     }
    //     else{
    //         oflags | (1 << 4) // O_WRONLY
    //     }
    // }
    // else {oflags};
    let new_flags = adjust_oflags(oflags, fs_rights_base);
    let r = wasi_path_open(ctx_ref, dirflags, path, path_len, new_flags, fdflags as i32);
    wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_readlink(ctx_ref, fd, path, path_len, buf, buf_len);
    wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_remove_directory(ctx_ref, fd, path, path_len);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
// TODO: pass through path_len
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_renameZ_iiiiiii(
    ctx: *const *mut VmCtx,
    old_fd: u32,
    old_path: u32,
    old_path_len: u32,
    new_fd: u32,
    new_path: u32,
    new_path_len: u32,
) -> u32 {
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
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_symlink(ctx_ref, old_path, old_path_len, fd, path, path_len);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_path_unlink_fileZ_iiii(
    ctx: *const *mut VmCtx,
    fd: u32,
    path: u32,
    path_len: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_path_unlink_file(ctx_ref, fd, path, path_len);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_poll_oneoff(ctx_ref, in_ptr, out_ptr, nsubscriptions);
    wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_proc_raiseZ_ii(
    ctx: *const *mut VmCtx,
    signal: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_proc_raise(ctx_ref, signal);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_random_getZ_iii(
    ctx: *const *mut VmCtx,
    buf: u32,
    buf_len: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_random_get(ctx_ref, buf, buf_len);
    wasm2c_marshal(r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sched_yieldZ_i(ctx: *const *mut VmCtx) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sched_yield(ctx_ref);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_recv(ctx_ref, fd, ri_data, ri_data_count, ri_flags);
    wasm2c_marshal_and_writeback_u32_pair(ctx_ref, out0 as usize, out1 as usize, r)
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
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_send(ctx_ref, fd, si_data, si_data_count, si_flags);
    wasm2c_marshal_and_writeback_u32(ctx_ref, out as usize, r)
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sock_shutdownZ_iii(
    ctx: *const *mut VmCtx,
    fd: u32,
    how: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_shutdown(ctx_ref, fd, how);
    wasm2c_marshal(r)
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
    let ctx_ref = ptr_to_ref(ctx);
    // let now = Instant::now();
    let r = wasi_socket(ctx_ref, domain, ty, protocol);
    wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r)
    // let final_result = wasm2c_marshal_and_writeback_u32(ctx_ref, retptr as usize, r);
    // println!("socket(): {:?}", now.elapsed());
    // final_result
}

#[no_mangle]
#[trace(logging)]
pub extern "C" fn Z_wasi_snapshot_preview1Z_sock_connectZ_iiii(
    ctx: *const *mut VmCtx,
    sockfd: u32,
    addr: u32,
    addrlen: u32,
) -> u32 {
    let ctx_ref = ptr_to_ref(ctx);
    let r = wasi_sock_connect(ctx_ref, sockfd, addr, addrlen);
    wasm2c_marshal(r)
}
