use crate::types::*;
use libc::{c_char, strlen};
use prusti_contracts::*;
use std::ffi::CStr;

/// To get wasm2c ffi working, we need to pass a VmCtx pointer back and forth
/// from C to Rust and back again.
/// The actual pointer that wasm2c gives us has a second layer of indrection
/// so we deref it twice to get the vmctx, then return a reference to that VmCtx
#[trusted]
pub fn ptr_to_ref(ctx: *const *mut VmCtx) -> &'static mut VmCtx {
    if ctx.is_null() {
        panic!("null ctx")
    }
    unsafe { &mut **ctx }
}

#[trusted]
pub fn transmut_netlist(nl: *const Netlist) -> Netlist {
    if nl.is_null() {
        panic!("null netlist")
    }
    unsafe { *nl }
}

#[trusted]
pub fn ffi_load_vec(ptr: *mut u8, len: usize) -> Vec<u8> {
    unsafe { Vec::from_raw_parts(ptr, len, len) }
}

#[trusted]
pub fn ffi_load_cstr(ptr: *const c_char) -> &'static str {
    unsafe { CStr::from_ptr(ptr).to_str().unwrap() }
}

#[trusted]
pub fn ffi_load_cstr_as_vec(ptr: *mut u8) -> Vec<u8> {
    let len = unsafe { strlen(ptr as *const i8) };
    unsafe { Vec::from_raw_parts(ptr, len, len) }
}
