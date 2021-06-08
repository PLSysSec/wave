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
        // 0 => {
        //     let pathname = __VERIFIER_nondet_unsigned_int();
        //     let fd = __VERIFIER_nondet_int();
        //     wasi_open(ctx, fd, pathname);
        // },
        1 => {
            let fd = unsafe{__VERIFIER_nondet_i32()};
            wasi_close(ctx, fd); 
        },
        // 2 => {
        //     let fd = __VERIFIER_nondet_int();
        //     let buf = __VERIFIER_nondet_unsigned_int();
        //     let size = __VERIFIER_nondet_unsigned_int();
        //     wasi_read(ctx, fd, buf, size);
        // },
        // 3 => {
        //     let fd = __VERIFIER_nondet_int();
        //     let buf = __VERIFIER_nondet_unsigned_int();
        //     let size = __VERIFIER_nondet_unsigned_int();
        //     wasi_write(ctx, fd, buf, size);
        // },
        _ => (),
    }
}


fn main() {
    let mut ctx = fresh_ctx();
    assert_safe(&ctx);
    make_one_syscall(&mut ctx);
    // assert_safe(&ctx);   
}
