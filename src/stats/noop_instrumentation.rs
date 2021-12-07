// noop functions useful in conditional compilation
// see tcb/os_specs/linux or wasm2c_frontend for details

#[inline]
pub fn start_timer() -> u64 {
    0
}

#[inline]
pub fn stop_timer() -> u64 {
    0
}

#[inline]
pub fn push_hostcall_result(_name: &str, _start: u64, _end: u64) {}

#[inline]
pub fn push_syscall_result(_name: &str, _start: u64, _end: u64) {}

pub fn output_hostcall_perf_results() {}
pub fn output_syscall_perf_results() {}
