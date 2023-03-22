#[cfg(feature = "time_syscalls")]
// use crate::stats::timing::{push_syscall_result, start_timer, stop_timer};
use crate::{iov::*, rvec::RVec};
use crate::{
    rvec::{BSlice, RVec},
    types::{NativeIoVec, SockAddr, VmCtx},
};
use libc::{c_int, mode_t, stat, timespec};
// use crate::tcb::misc::flag_set;
// use crate::tcb::sbox_mem::raw_ptr;
// use crate::tcb::verifier::*;
// use crate::types::NativeIoVec;
// #[cfg(not(feature = "time_syscalls"))]
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
// use crate::{effect, effects, path_effect};
// use prusti_contracts::*;
use syscall::syscall;
// use wave_macros::{external_call, external_method, with_ghost_var};

#[cfg_attr(target_os = "linux", path = "platform/linux.rs")]
#[cfg_attr(target_os = "macos", path = "platform/mac.rs")]
mod platform;
pub use platform::*;

pub use paste::paste;

use super::path::HostPath;

#[macro_export]
macro_rules! arg_converter {
    ($arg:ident: (&RVec<NativeIoVec>)) => {
        $arg.inner.as_ptr()
    };
    ($arg:ident: (&NativeIoVecs)) => {
        $arg.iovs.as_ptr()
    };
    ($arg:ident: HostPath) => {
        $arg.inner.as_ptr()
    };
    ($arg:ident: (&SockAddr)) => {
        &$arg.inner as *const libc::sockaddr_in
    };
    ($arg:ident: BSlice) => {
        $arg.inner.as_ptr()
    };
    ($arg:ident: (&[$type:ty])) => {
        $arg.as_ptr()
    };
    ($arg:ident: (&$type:ty)) => {
        $arg as *const $type
    };
    ($arg:ident: (&mut [$type:ty])) => {
        $arg.as_mut_ptr()
    };
    ($arg:ident: (&mut $type:ty)) => {
        $arg as *mut $type
    };
    ($arg:ident: [$type:ty; $size:expr]) => {
        $arg.as_ptr()
    };
    ($arg:ident: $type:ty) => {
        $arg
    };
}

#[macro_export]
macro_rules! syscall_spec_gen {
    {   sig($sig:meta);
        syscall($name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            // #[with_ghost_var($tr: &mut Trace)]
            // $(#[requires($pre)])*
            // $(#[ensures($post)])*
            #[flux::trusted]
            #[$sig]
            pub fn [<os_ $name>]($($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall::syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    { syscall($name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[flux::trusted]
            pub fn [<os_ $name>]($($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall::syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    {   // $tr:ident;
        // $(requires($pre:tt);)*
        // $(ensures($post:tt);)*
        syscall($name:ident ALIAS $os_name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[flux::trusted]
            // #[with_ghost_var($tr: &mut Trace)]
            // $(#[requires($pre)])*
            // $(#[ensures($post)])*
            pub fn [<os_ $os_name>]($($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    {   // $tr:ident;
        // $(requires($pre:tt);)*
        // $(ensures($post:tt);)*
        sig($sig:meta);
        syscall($name:ident ALIAS $os_name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[flux::trusted]
            #[$sig]
            // #[with_ghost_var($tr: &mut Trace)]
            // $(#[requires($pre)])*
            // $(#[ensures($post)])*
            pub fn [<os_ $os_name>]($($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall::syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    // syscall_with_cx (duplicate here)
    {   sig($sig:meta);
        syscall_with_cx($name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            // #[with_ghost_var($tr: &mut Trace)]
            // $(#[requires($pre)])*
            // $(#[ensures($post)])*
            #[flux::trusted]
            #[$sig]
            pub fn [<os_ $name>](_cx: &VmCtx, $($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall::syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    { syscall_with_cx($name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[flux::trusted]
            pub fn [<os_ $name>](_cx: &VmCtx, $($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    {   // $tr:ident;
        // $(requires($pre:tt);)*
        // $(ensures($post:tt);)*
        syscall_with_cx($name:ident ALIAS $os_name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[flux::trusted]
            // #[with_ghost_var($tr: &mut Trace)]
            // $(#[requires($pre)])*
            // $(#[ensures($post)])*
            pub fn [<os_ $os_name>](_cx: &VmCtx, $($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    {   // $tr:ident;
        // $(requires($pre:tt);)*
        // $(ensures($post:tt);)*
        sig($sig:meta);
        syscall_with_cx($name:ident ALIAS $os_name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[flux::trusted]
            #[$sig]
            // #[with_ghost_var($tr: &mut Trace)]
            // $(#[requires($pre)])*
            // $(#[ensures($post)])*
            pub fn [<os_ $os_name>](_cx: &VmCtx, $($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall::syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    }
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::O_NOFOLLOW))));
    sig(flux::sig(fn(ctx: &VmCtx[@cx], dirfd: usize, path: HostPathSafe[!flag_set(flags, O_NOFOLLOW)], flags: i32, mode: i32) -> isize requires PathAccessAt(dirfd, cx.homedir_host_fd)));
    syscall_with_cx(openat, dirfd: usize, path: HostPath, flags: i32, mode: i32)
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    sig(flux::sig(fn(ctx: &VmCtx, fd: usize) -> isize));
    syscall_with_cx(close, fd: usize)
}

syscall_spec_gen! {
    // trace;
    // requires((buf.len() >= cnt));
    // ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    // ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    sig(flux::sig(fn(fd: usize, buf: BSlice, cnt: usize{cnt <= buf.len}) -> isize
                    requires WriteMem(buf.base, buf.addr, cnt)));
    syscall(read, fd: usize, buf: BSlice, cnt: usize)
}

syscall_spec_gen! {
    // trace;
    // ensures((trace.len() == old(trace.len() + buf.len())
    // && forall(|i: usize| (i < trace.len()) ==>
    // {
    //     if i < old(trace.len())
    //         { trace.lookup(i) == old(trace.lookup(i)) }
    //     else
    //     {
    //         let this = buf.lookup(i - old(trace.len()));
    //         let ev = trace.lookup(i);
    //         iov_eq_write(ev, &this)
    //     }
    // }
    // ));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], fd: usize, buf: &RVec<NativeIoVecOk[cx.base]>, iovcnt: usize) -> isize));
    syscall_with_cx(readv, fd: usize, buf: (&RVec<NativeIoVec>), iovcnt: usize)
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(ReadMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    sig(flux::sig(fn(fd: usize, buf: BSlice, cnt: usize{cnt <= buf.len}) -> isize
                    requires ReadMem(buf.base, buf.addr, cnt)));
    syscall(write, fd: usize, buf: BSlice, cnt: usize)
}

syscall_spec_gen! {
    // trace;
    // ensures((
    //     trace.len() == old(trace.len() + buf.len()) &&
    //     forall(|i: usize| (i < trace.len()) ==>
    //         {
    //             if i < old(trace.len())
    //                 { trace.lookup(i) == old(trace.lookup(i)) }
    //             else
    //             {
    //                 let this = buf.lookup(i - old(trace.len()));
    //                 let ev = trace.lookup(i);
    //                 iov_eq_read(ev, &this)
    //             }
    //         }
    //     )
    // ));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], fd: usize, buf: &RVec<NativeIoVecOk[cx.base]>, iovcnt: usize) -> isize));
    syscall_with_cx(writev, fd: usize, buf: (&RVec<NativeIoVec>), iovcnt: usize)
}

syscall_spec_gen! {
    // trace;
    // ensures((
    //     trace.len() == old(trace.len() + buf.len()) &&
    //     forall(|i: usize| (i < trace.len()) ==>
    //         {
    //             if i < old(trace.len())
    //                 { trace.lookup(i) == old(trace.lookup(i)) }
    //             else
    //             {
    //                 let this = buf.lookup(i - old(trace.len()));
    //                 let ev = trace.lookup(i);
    //                 iov_eq_write(ev, &this)
    //             }
    //         }
    //     )
    // ));

    sig(flux::sig(fn (ctx: &VmCtx[@cx], fd: usize, buf: &RVec<NativeIoVecOk[cx.base]>, iovcnt: usize, offset: usize) -> isize));
    syscall_with_cx(preadv, fd: usize, buf: (&RVec<NativeIoVec>), iovcnt: usize, offset: usize)
}

syscall_spec_gen! {
    // trace;
    // ensures((
    //     trace.len() == old(trace.len() + buf.len()) &&
    //     forall(|i: usize| (i < trace.len()) ==>
    //         {
    //             if i < old(trace.len())
    //                 { trace.lookup(i) == old(trace.lookup(i)) }
    //             else
    //             {
    //                 let this = buf.lookup(i - old(trace.len()));
    //                 let ev = trace.lookup(i);
    //                 iov_eq_read(ev, &this)
    //             }
    //         }
    //     )
    // ));

    sig(flux::sig(fn (ctx: &VmCtx[@cx], fd: usize, buf: &RVec<NativeIoVecOk[cx.base]>, iovcnt: usize, offset: usize) -> isize));
    syscall_with_cx(pwritev, fd: usize, buf: (&RVec<NativeIoVec>), iovcnt: usize, offset: usize)
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], fd: usize, offset: i64, whence: i32) -> isize));
    syscall_with_cx(lseek, fd: usize, offset: i64, whence: i32)
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    sig(flux::sig(fn (fd: usize) -> isize));
    syscall(sync, fd: usize)
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    sig(flux::sig(fn (fd: usize) -> isize));
    syscall(fdatasync, fd: usize)
}

// TODO: for very broad syscalls might just want to keep them manually
//       written to reduce the expsure to unsafe
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    sig(flux::sig(fn (fd: usize, cmd: i32, arg: c_int) -> isize requires Shutdown() && FdAccess()));
    syscall(fcntl, fd: usize, cmd: i32, arg: c_int)
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(ftruncate, fd: usize, length: (libc::off_t))
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess),
    //     effect!(FdAccess),
    //     path_effect!(PathAccessAt, fd1, old_p, f) if fd1 == old_fd && old_p == old(old_path) && f == flag_set(flags, libc::AT_SYMLINK_FOLLOW),
    //     path_effect!(PathAccessAt, fd2, new_p, f) if fd2 == new_fd && new_p == old(new_path) && f == flag_set(flags, libc::AT_SYMLINK_FOLLOW)
    // )));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], old_fd: usize, old_path: HostPathSafe[flag_set(flags, AT_SYMLINK_FOLLOW)], new_fd: usize, new_path: HostPathSafe[flag_set(flags, AT_SYMLINK_FOLLOW)], flags: i32) -> isize
                  requires FdAccess() && PathAccessAt(old_fd, cx.homedir_host_fd) && PathAccessAt(new_fd, cx.homedir_host_fd)));
    syscall_with_cx(linkat, old_fd: usize, old_path: HostPath, new_fd: usize, new_path: HostPath, flags: i32)
}

// https://man7.org/linux/man-pages/man2/mkdirat.2.html
// follows terminal symlink: true
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, true) if fd == dirfd && p == old(path) )));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], dirfd: usize, path: HostPathSafe[true], mode: mode_t) -> isize requires FdAccess() && PathAccessAt(dirfd, cx.homedir_host_fd)));
    syscall_with_cx(mkdirat, dirfd: usize, path: HostPath, mode: mode_t)
}

// https://man7.org/linux/man-pages/man2/readlinkat.2.html
// follows terminal symlink: false
syscall_spec_gen! {
    // trace;
    // requires((buf.len() >= cnt));
    // ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    // ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, false) if fd == dirfd && p == old(path), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    // FLUX-TODO: cannot pass in VmCtx due to BSlice ownership, hence no precond on dirfd: usize[cx.homedir_host_fd]
    sig(flux::sig(fn (dirfd: usize, path: HostPathSafe[false], buf: BSlice, cnt: usize{buf.len >= cnt }) -> isize requires FdAccess() && WriteMem(buf.base, buf.addr, cnt)));
    syscall(readlinkat, dirfd: usize, path: HostPath, buf: BSlice, cnt: usize)
}

// https://man7.org/linux/man-pages/man2/unlinkat.2.html
// follows terminal symlink: false
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, false) if fd == dirfd && p == old(path))));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], dirfd: usize, path: HostPathSafe[false], flags: c_int) -> isize requires FdAccess() && PathAccessAt(dirfd, cx.homedir_host_fd)));
    syscall_with_cx(unlinkat, dirfd: usize, path: HostPath, flags: c_int)
}

//https://man7.org/linux/man-pages/man2/renameat.2.html
// follows terminal symlinks: false
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd1, old_p, false) if fd1 == old_dir_fd && old_p == old(old_path), effect!(FdAccess), path_effect!(PathAccessAt, fd2, new_p, false) if fd2 == new_dir_fd && new_p == old(new_path))));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], old_dir_fd: usize, old_path: HostPathSafe[false], new_dir_fd: usize, new_path: HostPathSafe[false]) -> isize requires FdAccess() && PathAccessAt(old_dir_fd, cx.homedir_host_fd) && PathAccessAt(new_dir_fd, cx.homedir_host_fd)));
    syscall_with_cx(renameat, old_dir_fd: usize, old_path: HostPath, new_dir_fd: usize, new_path: HostPath)
}

// https://man7.org/linux/man-pages/man2/symlinkat.2.html
// From the spec: The string pointed to by path1 shall be treated only as a string and shall not be validated as a pathname.
// follows terminal symlinks: true (although it might fail)
// TODO: do we actually need to check the second path or can we just let the resolver do its thing?
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, path_effect!(PathAccessAt, fd, p, true) if fd == dirfd && p == old(path2), effect!(FdAccess))));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], path1: HostPathSafe[true], dirfd: usize, path2: HostPathSafe[true]) -> isize requires FdAccess() && PathAccessAt(dirfd, cx.homedir_host_fd)));
    syscall_with_cx(symlinkat, path1: HostPath, dirfd: usize, path2: HostPath)
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
syscall_spec_gen! {
    // trace;
    // requires((buf.len() >= cnt));
    // ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    // ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    sig(flux::sig(fn (fd: usize, buf: BSlice, cnt: usize{buf.len >= cnt}, flags: i32, src: i32, addrlen: i32) -> isize requires FdAccess() && WriteMem(buf.base, buf.addr, cnt)));
    syscall(recvfrom, fd: usize, buf: BSlice, cnt: usize, flags: i32, src: i32, addrlen: i32)
}

//https://man7.org/linux/man-pages/man2/sendto.2.html
syscall_spec_gen! {
    // trace;
    // requires((buf.len() >= cnt));
    // ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(ReadMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    sig(flux::sig(fn (fd: usize, buf: BSlice, cnt: usize{buf.len >= cnt}, flags: i32, dest_addr: i32, addrlen: i32) -> isize requires FdAccess() && ReadMem(buf.base, buf.addr, cnt)));
    syscall(sendto, fd: usize, buf: BSlice, cnt: usize, flags: i32, dest_addr: i32, addrlen: i32)
}

//https://man7.org/linux/man-pages/man2/shutdown.2.html
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(Shutdown), effect!(FdAccess))));
    syscall(shutdown, fd: usize, how: (libc::c_int))
}

//https://man7.org/linux/man-pages/man2/poll.2.html
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(poll, pollfds: (&mut RVec<libc::pollfd>), timeout: (libc::c_int))
}

//https://man7.org/linux/man-pages/man2/socket.2.html
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(SockCreation, d, t) if d == (domain as usize) && t == (ty as usize) )));
    sig(flux::sig(fn (domain: i32, ty: i32, protocol: i32) -> isize requires SockCreation(domain, ty)));
    syscall(socket, domain: i32, ty: i32, protocol: i32)
}

//https://man7.org/linux/man-pages/man2/connect.2.html
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(NetAccess, protocol, ip, port) if ip == addr.sin_addr.s_addr as usize && port == addr.sin_port as usize)));
    sig(flux::sig(fn (ctx: &VmCtx[@cx], sockfd: usize, addr: &SockAddr[@saddr], addrlen: u32) -> isize requires FdAccess() && NetAccess(cx.net, saddr.addr, saddr.port)));
    syscall_with_cx(connect, sockfd: usize, addr: (&SockAddr), addrlen: u32)
}

//https://man7.org/linux/man-pages/man2/ioctl.2.html
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(ioctl, fd: usize, request: (libc::c_ulong))
}
