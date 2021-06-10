extern crate smack;
#[macro_use]
extern crate syscall;
pub mod runtime;
pub mod os;
pub mod wrappers;

use smack::*;
use runtime::*;
use wrappers::*;

// Harness for verifier
//1. Create a fresh sandbox context
//2. Invoke system calls
fn make_one_syscall(ctx: &mut VmCtx){
    // requires(SAFE(ctx));
    // ensures(SAFE(ctx));
    let choice = unsafe{__VERIFIER_nondet_u32()};
    match choice {
        0 => {
            let pathname = unsafe{__VERIFIER_nondet_u32()};
            let flags = unsafe{__VERIFIER_nondet_i32()};
            wasi_open(ctx, pathname, flags);
        },
        1 => {
            let fd = unsafe{__VERIFIER_nondet_i32()};
            wasi_close(ctx, fd); 
        },
        2 => {
            let fd = unsafe{__VERIFIER_nondet_i32()};
            let buf = unsafe{__VERIFIER_nondet_u32()};
            let size = unsafe{__VERIFIER_nondet_u32()};
            wasi_read(ctx, fd, buf, size as usize);
        },
        3 => {
            let fd = unsafe{__VERIFIER_nondet_i32()};
            let buf = unsafe{__VERIFIER_nondet_u32()};
            let size = unsafe{__VERIFIER_nondet_u32()};
            wasi_write(ctx, fd, buf, size as usize);
        },
        _ => (),
    }
}


fn main() {
    let mut ctx = fresh_ctx();
    assert_safe(&ctx);
    make_one_syscall(&mut ctx);
    // assert_safe(&ctx);   
}
