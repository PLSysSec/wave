use dlopen::wrapper::{Container, WrapperApi};
use wave::types::{Netlist, VmCtx};


#[derive(Debug)]
pub struct WaveConfig {
    pub module_path: String,
    pub homedir: String,
    pub netlist: Netlist,
    pub args: Vec<u8>,
    pub argc: usize,
    pub env: Vec<u8>,
    pub envc: usize,
}

// #[derive(Debug)]
pub struct WaveSandbox {
    pub module: Container<Wasm2cBinary>,
    pub vmctx: VmCtx,
    pub linmem: *mut u8,
}

#[derive(WrapperApi, Debug)]
pub struct Wasm2cBinary {
    w2c__start: unsafe extern "C" fn() -> i32,
}

