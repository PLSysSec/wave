use crate::iov::*;
#[cfg(feature = "time_syscalls")]
use crate::stats::timing::{push_syscall_result, start_timer, stop_timer};
use crate::tcb::misc::flag_set;
use crate::tcb::sbox_mem::raw_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::{NativeIoVec, NativeIoVecs};
#[cfg(not(feature = "time_syscalls"))]
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, effects, path_effect};
use prusti_contracts::*;
use syscall::syscall;
use wave_macros::{external_call, external_method, with_ghost_var};

#[cfg_attr(target_os = "linux", path = "platform/linux.rs")]
#[cfg_attr(target_os = "macos", path = "platform/mac.rs")]
mod platform;
pub use platform::*;

pub use paste::paste;

#[macro_export]
macro_rules! arg_converter {
    ($arg:ident: (&NativeIoVecs)) => {
        $arg.iovs.as_ptr()
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
    {   $tr:ident;
        $(requires($pre:tt);)*
        $(ensures($post:tt);)*
        syscall($name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[with_ghost_var($tr: &mut Trace)]
            #[trusted]
            $(#[requires($pre)])*
            $(#[ensures($post)])*
            pub fn [<os_ $name>]($($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    };
    {   $tr:ident;
        $(requires($pre:tt);)*
        $(ensures($post:tt);)*
        syscall($name:ident ALIAS $os_name:ident, $($arg:ident: $type:tt),*)
    } => {
        paste! {
            #[with_ghost_var($tr: &mut Trace)]
            #[trusted]
            $(#[requires($pre)])*
            $(#[ensures($post)])*
            pub fn [<os_ $os_name>]($($arg: $type),*) -> isize {
                use $crate::arg_converter;
                let __start_ts = start_timer();
                let result = unsafe { syscall!([<$name:upper>], $(arg_converter!($arg: $type)),*) as isize };
                let __end_ts = stop_timer();
                push_syscall_result(stringify!($name), __start_ts, __end_ts);
                return result;
            }
        }
    }
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::O_NOFOLLOW))));
    syscall(openat, dirfd: usize, path: [u8; 4096], flags: i32, mode: i32)
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(close, fd: usize)
}

syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    syscall(read, fd: usize, buf: (&mut [u8]), cnt: usize)
}

syscall_spec_gen! {
    trace;
    ensures((trace.len() == old(trace.len() + buf.len()) &&
    forall(|i: usize| (i < trace.len()) ==>
    {
        if i < old(trace.len())
            { trace.lookup(i) == old(trace.lookup(i)) }
        else
        {
            let this = buf.lookup(i - old(trace.len()));
            let ev = trace.lookup(i);
            iov_eq_write(ev, &this)
        }
    }
    )));
    syscall(readv, fd: usize, buf: (&NativeIoVecs), iovcnt: usize)
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(ReadMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    syscall(write, fd: usize, buf: (&[u8]), cnt: usize)
}

syscall_spec_gen! {
    trace;
    ensures((
        trace.len() == old(trace.len() + buf.len()) &&
        forall(|i: usize| (i < trace.len()) ==>
            {
                if i < old(trace.len())
                    { trace.lookup(i) == old(trace.lookup(i)) }
                else
                {
                    let this = buf.lookup(i - old(trace.len()));
                    let ev = trace.lookup(i);
                    iov_eq_read(ev, &this)
                }
            }
        )
    ));
    syscall(writev, fd: usize, buf: (&NativeIoVecs), iovcnt: usize)
}

syscall_spec_gen! {
    trace;
    ensures((
        trace.len() == old(trace.len() + buf.len()) &&
        forall(|i: usize| (i < trace.len()) ==>
            {
                if i < old(trace.len())
                    { trace.lookup(i) == old(trace.lookup(i)) }
                else
                {
                    let this = buf.lookup(i - old(trace.len()));
                    let ev = trace.lookup(i);
                    iov_eq_write(ev, &this)
                }
            }
        )
    ));
    syscall(preadv, fd: usize, buf: (&NativeIoVecs), iovcnt: usize, offset: usize)
}

syscall_spec_gen! {
    trace;
    ensures((
        trace.len() == old(trace.len() + buf.len()) &&
        forall(|i: usize| (i < trace.len()) ==>
            {
                if i < old(trace.len())
                    { trace.lookup(i) == old(trace.lookup(i)) }
                else
                {
                    let this = buf.lookup(i - old(trace.len()));
                    let ev = trace.lookup(i);
                    iov_eq_read(ev, &this)
                }
            }
        )
    ));
    syscall(pwritev, fd: usize, buf: (&NativeIoVecs), iovcnt: usize, offset: usize)
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(lseek, fd: usize, offset: i64, whence: i32)
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(sync, fd: usize)
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(fdatasync, fd: usize)
}

// TODO: for very broad syscalls might just want to keep them manually
//       written to reduce the expsure to unsafe
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(fcntl, fd: usize, cmd: i32, arg: (libc::c_int))
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(ftruncate, fd: usize, length: (libc::off_t))
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess),
        effect!(FdAccess),
        path_effect!(PathAccessAt, fd1, old_p, f) if fd1 == old_fd && old_p == old(old_path) && f == flag_set(flags, libc::AT_SYMLINK_FOLLOW),
        path_effect!(PathAccessAt, fd2, new_p, f) if fd2 == new_fd && new_p == old(new_path) && f == flag_set(flags, libc::AT_SYMLINK_FOLLOW)
    )));
    syscall(linkat, old_fd: usize, old_path: [u8; 4096], new_fd: usize, new_path: [u8; 4096], flags: i32)
}

// https://man7.org/linux/man-pages/man2/mkdirat.2.html
// follows terminal symlink: true
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, true) if fd == dirfd && p == old(path) )));
    syscall(mkdirat, dirfd: usize, path: [u8; 4096], mode: (libc::mode_t))
}

// https://man7.org/linux/man-pages/man2/readlinkat.2.html
// follows terminal symlink: false
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, false) if fd == dirfd && p == old(path), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    syscall(readlinkat, dirfd: usize, path: [u8; 4096], buf: (&mut [u8]), cnt: usize)
}

// https://man7.org/linux/man-pages/man2/unlinkat.2.html
// follows terminal symlink: false
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, false) if fd == dirfd && p == old(path))));
    syscall(unlinkat, dirfd: usize, path: [u8; 4096], flags: (libc::c_int))
}

//https://man7.org/linux/man-pages/man2/renameat.2.html
// follows terminal symlinks: false
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd1, old_p, false) if fd1 == old_dir_fd && old_p == old(old_path), effect!(FdAccess), path_effect!(PathAccessAt, fd2, new_p, false) if fd2 == new_dir_fd && new_p == old(new_path))));
    syscall(renameat, old_dir_fd: usize, old_path: [u8; 4096], new_dir_fd: usize, new_path: [u8; 4096])
}

// https://man7.org/linux/man-pages/man2/symlinkat.2.html
// From the spec: The string pointed to by path1 shall be treated only as a string and shall not be validated as a pathname.
// follows terminal symlinks: true (although it might fail)
// TODO: do we actually need to check the second path or can we just let the resolver do its thing?
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, path_effect!(PathAccessAt, fd, p, true) if fd == dirfd && p == old(path2), effect!(FdAccess))));
    syscall(symlinkat, path1: [u8; 4096], dirfd: usize, path2: [u8; 4096])
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    syscall(recvfrom, fd: usize, buf: (&mut [u8]), cnt: usize, flags: i32, src: i32, addrlen: i32)
}

//https://man7.org/linux/man-pages/man2/sendto.2.html
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(ReadMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    syscall(sendto, fd: usize, buf: (&[u8]), cnt: usize, flags: i32, dest_addr: i32, addrlen: i32)
}

//https://man7.org/linux/man-pages/man2/shutdown.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(Shutdown), effect!(FdAccess))));
    syscall(shutdown, fd: usize, how: (libc::c_int))
}

//https://man7.org/linux/man-pages/man2/poll.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(poll, pollfds: (&mut [libc::pollfd]), timeout: (libc::c_int))
}

//https://man7.org/linux/man-pages/man2/socket.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(SockCreation, d, t) if d == (domain as usize) && t == (ty as usize) )));
    syscall(socket, domain: i32, ty: i32, protocol: i32)
}

//https://man7.org/linux/man-pages/man2/connect.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(NetAccess, protocol, ip, port) if ip == addr.sin_addr.s_addr as usize && port == addr.sin_port as usize)));
    syscall(connect, sockfd: usize, addr: (&libc::sockaddr_in), addrlen: u32)
}

//https://man7.org/linux/man-pages/man2/ioctl.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(ioctl, fd: usize, request: (libc::c_ulong))
}
