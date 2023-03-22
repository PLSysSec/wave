use std::ptr::copy_nonoverlapping;

use crate::rvec::{BSlice, RVec};
use crate::types::{NativeIoVec, SboxPtr, VmCtx, WasmIoVec};

impl VmCtx {
    #[flux::trusted]
    #[flux::sig(fn(self: &VmCtx[@cx], WasmIoVec) -> NativeIoVecOk[cx.base])]
    pub fn translate_iov(&self, iov: WasmIoVec) -> NativeIoVec {
        // let swizzled_base = self.raw + iov.iov_base as usize;
        let swizzled_base =
            unsafe { self.mem.inner.as_ptr().offset(iov.iov_base as isize) as usize };
        NativeIoVec {
            iov_base: swizzled_base,
            iov_len: iov.iov_len as usize,
        }
    }

    // FLUX-TODO2: capacity (see wave spec)
    #[flux::trusted]
    #[flux::sig(fn(self: &VmCtx, &mut RVec<u8>[n], src: SboxPtr{src + n < LINEAR_MEM_SIZE}, n:u32{0 <= n}))]
    #[allow(unused_variables)]
    pub fn memcpy_from_sandbox(&self, dst: &mut RVec<u8>, src: SboxPtr, n: u32) {
        unsafe {
            copy_nonoverlapping(
                self.mem.inner.as_ptr().offset(src as isize),
                dst.inner.as_mut_ptr(),
                n as usize,
            );
            dst.set_len(n as usize); // TODO: wrong, need to make sure copy_nonoverlapping actually copied it
        };
        // do_effect!(effect!(ReadMem, src, n));
    }

    #[allow(unused_variables)]
    #[flux::trusted]
    #[flux::sig(fn(self: &mut VmCtx[@cx], dst: SboxPtr{dst + n < LINEAR_MEM_SIZE}, &RVec<u8>{sz:n <= sz}, n:u32))]
    pub fn memcpy_to_sandbox(&mut self, dst: SboxPtr, src: &RVec<u8>, n: u32) {
        unsafe {
            copy_nonoverlapping(
                src.inner.as_ptr(),
                self.mem.inner.as_mut_ptr().offset(dst as isize),
                n as usize,
            )
        };
    }

    // Currently trusted because it causes a fold-unfold error
    // #[with_ghost_var(trace: &mut Trace)]
    // #[requires(self.fits_in_lin_mem(ptr, len, trace))]
    // #[requires(trace_safe(trace, self))]
    // #[ensures(result.len() == (len as usize))]
    // #[ensures(effects!(old(trace), trace))]
    // #[ensures(raw_ptr(result) == old(raw_ptr(self.mem.as_slice())) + ptr as usize)]
    // #[after_expiry(
    //     ctx_safe(self) &&
    //     old(raw_ptr(self.mem.as_slice()) + ptr as usize) == before_expiry(raw_ptr(result)) &&
    //     raw_ptr(self.mem.as_slice()) + ptr as usize == before_expiry(raw_ptr(result)) &&
    //     old(self.netlist) == self.netlist &&
    //     old(self.homedir_host_fd) == self.homedir_host_fd)]
    #[flux::trusted]
    #[flux::sig(fn(self: &mut VmCtx[@cx], ptr: SboxPtr, len: u32{fits_in_lin_mem(ptr, len)}) -> &mut [u8][len])]
    pub fn slice_mem_mut(&mut self, ptr: SboxPtr, len: u32) -> &mut [u8] {
        let start = ptr as usize;
        let end = ptr as usize + len as usize;
        &mut self.mem.inner[start..end]
    }

    #[flux::trusted]
    #[flux::sig(fn(self: &mut VmCtx[@cx], ptr: SboxPtr, len: u32{fits_in_lin_mem(ptr, len)}) -> BSlice[cx.base, cx.base + ptr, len])]
    pub fn rslice_mem_mut(&mut self, ptr: SboxPtr, len: u32) -> BSlice {
        let start = ptr as usize;
        let end = ptr as usize + len as usize;
        BSlice {
            inner: &mut self.mem.inner[start..end],
        }
    }
}
