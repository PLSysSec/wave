use crate::types::{Wasm2cBinary, WaveConfig, WaveSandbox};
use dlopen::wrapper::Container;
use libc::{mprotect, munmap};
use std::ptr;
use wave::wasm2c_frontend::{create_ctx, wave_cleanup}; // TODO: fix path in wave
use wave::types::VmCtx;
use wave::setup_teardown::wave_alloc_linmem;
// handles wasm2c files

// // TODO: how does the stack guard work?
// // TODO: direct syscalls for mmap, munmap, and mprotect?
// // TODO: zero initialize linear memory



// 1 << 32 = 4GB
const FOUR_GB: usize = 1 << 32;
// 1 << 33 = 8GB
const EIGHT_GB: usize = 1 << 33;



fn wave_setup_signals() {
    // For now, do nothings, since we just fail on any signal
}

// Setup has 3 steps
// 1. load the AOT-compiled Wasm module via DlOpen
// 2. MMap and mprotect the linear memory
// 3. Setup signals (maybe?)
// It then returns a wave_runtime object with all these things set up
fn setup(config: &WaveConfig) -> WaveSandbox {
    // 1. Load AOT-compiled Wasm module
    let module: Container<Wasm2cBinary> = unsafe {
        Container::load(config.module_path.as_str()) // wrapper around dlopen
    }
    .unwrap();

    // 2. MMap and mprotect the linear memory
    let linmem = wave_alloc_linmem() as *mut u8;

    // 3. Setup signals
    wave_setup_signals();

    let vmctx = create_ctx(
        linmem,
        &config.homedir,
        config.args.clone(),
        config.argc,
        config.env.clone(),
        config.envc,
        config.netlist,
    );

    WaveSandbox {
        module,
        vmctx,
        linmem,
    }
}

// The execute stage simply consists of calling the __start function
// returns the result code of executing the sandbox
fn execute(sandbox: &WaveSandbox) -> i32 {
    unsafe { sandbox.module.w2c__start() }
}

// // Here, we just:
// // 1. Unmap linear memory
// // 2. Drop the sandbox
fn teardown(mut sandbox: WaveSandbox) {
    unsafe {
    // release file descriptors and such
    wave_cleanup(&(&mut sandbox.vmctx as *mut VmCtx) as *const *mut VmCtx);
    // unmap linear memory
    
        munmap(sandbox.linmem as *mut libc::c_void, EIGHT_GB);
    }
    // then the sandbox gets dropped automatically by rustc since we moved the sandbox here
}

// Run consists of 3 steps:
// 1. Set up runtime
// 2. Execute AOT-compiled Wasm binary
// 3. Teardown runtime
pub fn run(config: &WaveConfig) {
    let sandbox = setup(config);
    let _result = execute(&sandbox);
    teardown(sandbox);
}

