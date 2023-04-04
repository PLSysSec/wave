use crate::{
    path_resolution::resolve_path,
    rvec::RVec,
    tcb::{
        misc::{empty_netlist, get_homedir_fd, string_to_rvec_u8},
        path::HostPathSafe,
    },
    types::*,
};
use RuntimeError::*;

#[flux::alias(type FitsBool(buf: int, cnt: int) = bool[fits_in_lin_mem(buf, cnt)])]
pub type FitsBool = bool;

#[flux::alias(type FitsUsize(buf: int) = usize{cnt : fits_in_lin_mem(buf, cnt)})]
pub type FitsUsize = usize;

//#[ensures(safe(&result))]
// #[with_ghost_var(trace: &mut Trace)]
// #[external_methods(init_std_fds, unwrap, as_raw_fd, create, to_owned, clone)]
// #[external_calls(open, forget, get_homedir_fd, from)]
pub fn fresh_ctx(homedir: String) -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    // let mem = vec![0; memlen];
    let mem = RVec::from_elem_n(0, memlen);

    let mut fdmap = FdMap::new();
    fdmap.init_std_fds();
    let homedir_host_fd = get_homedir_fd(&homedir) as usize;
    // let homedir_file = std::fs::File::open(&homedir).unwrap();
    // let homedir_fd = homedir_file.as_raw_fd();
    if homedir_host_fd >= 0 {
        fdmap.create(HostFd::from_raw(homedir_host_fd));
    }
    // Need to forget file to make sure it does not get auto-closed
    // when it gets out of scope
    // std::mem::forget(homedir_file);
    // let log_path = "".to_owned();
    // let log_path = String::new();

    let arg_buffer = RVec::new();
    let argc = 0;
    let env_buffer = RVec::new();
    let envc = 0;

    let netlist = empty_netlist();
    VmCtx {
        ghost_raw: 100, //
        mem,
        memlen,
        fdmap,
        homedir,
        homedir_host_fd: HostFd::from_raw(homedir_host_fd),
        // errno: Success,
        arg_buffer,
        argc,
        env_buffer,
        envc,
        // log_path,
        netlist,
    }
}

impl VmCtx {
    /// Check whether sandbox pointer is actually inside the sandbox
    // TODO: can I eliminate this in favor os in_lin_mem_usize?
    #[flux::sig(fn(&VmCtx, ptr:SboxPtr) -> bool[0 <= ptr && ptr < LINEAR_MEM_SIZE])]
    pub fn in_lin_mem(&self, ptr: SboxPtr) -> bool {
        (ptr as usize >= 0) && (ptr as usize) < self.memlen
    }

    #[flux::sig(fn(&VmCtx, ptr:usize) -> bool[0 <= ptr && ptr < LINEAR_MEM_SIZE])]
    pub fn in_lin_mem_usize(&self, ptr: usize) -> bool {
        ptr >= 0 && ptr < self.memlen
    }

    /// Check whether buffer is entirely within sandbox
    // Can I eliminate this in favor of fits_in_lin_mem_usize
    #[flux::sig(fn(&VmCtx, buf: SboxPtr, cnt:u32) -> FitsBool(buf, cnt))]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: u32) -> FitsBool {
        let total_size = (buf as usize) + (cnt as usize);
        if total_size >= self.memlen {
            return false;
        }
        self.in_lin_mem(buf) && self.in_lin_mem(cnt) && buf <= buf + cnt
    }

    #[flux::sig(fn(&VmCtx, buf:usize, cnt:usize) -> FitsBool(buf, cnt))]
    pub fn fits_in_lin_mem_usize(&self, buf: usize, cnt: usize) -> FitsBool {
        let total_size = buf + cnt;
        if total_size >= self.memlen {
            return false;
        }
        self.in_lin_mem_usize(buf) && self.in_lin_mem_usize(cnt) && buf <= buf + cnt
    }

    /// Copy buffer from sandbox to host
    #[flux::sig(fn(&VmCtx, src:SboxPtr, n:u32{0 <= n && src + n < LINEAR_MEM_SIZE}) -> RVec<u8>[n])]
    pub fn copy_buf_from_sandbox(&self, src: SboxPtr, n: u32) -> RVec<u8> {
        let mut host_buffer: RVec<u8> = RVec::from_elem_n(0, n as usize);
        // FLUX-TODO2: capacity: host_buffer.reserve_exact(n as usize);
        // assert!(src >= 0);
        // assert!(((n as usize) < self.memlen) && ((n as usize) >= 0));
        self.memcpy_from_sandbox(&mut host_buffer, src, n);
        host_buffer
    }

    /// Copy buffer from from host to sandbox
    #[flux::sig(fn(self: &mut VmCtx[@dummy], SboxPtr, &RVec<u8>, u32) -> Result<(), RuntimeError>)]
    pub fn copy_buf_to_sandbox(
        &mut self,
        dst: SboxPtr,
        src: &RVec<u8>,
        n: u32,
    ) -> Result<(), RuntimeError> {
        if src.len() < n as usize || !self.fits_in_lin_mem(dst, n) {
            return Err(Efault);
        }
        self.memcpy_to_sandbox(dst, src, n);
        Ok(())
    }

    /// Copy arg buffer from from host to sandbox
    #[flux::sig(fn(&mut {VmCtx[@cx] : cx.arg_buf == n}, dst: SboxPtr, n:u32) -> Result<(), RuntimeError>)]
    pub fn copy_arg_buffer_to_sandbox(&mut self, dst: SboxPtr, n: u32) -> Result<(), RuntimeError> {
        if !self.fits_in_lin_mem(dst, n) {
            return Err(Efault);
        }
        let arg_buffer = &self.arg_buffer.clone();
        self.memcpy_to_sandbox(dst, &arg_buffer, n);
        Ok(())
    }

    /// Copy arg buffer from from host to sandbox
    #[flux::sig(fn(&mut {VmCtx[@cx] : cx.env_buf == n}, dst: SboxPtr, n:u32) -> Result<(), RuntimeError>)]
    pub fn copy_environ_buffer_to_sandbox(
        &mut self,
        dst: SboxPtr,
        n: u32,
    ) -> Result<(), RuntimeError> {
        if !self.fits_in_lin_mem(dst, n) {
            return Err(Efault);
        }
        let env_buffer = &self.env_buffer.clone();
        self.memcpy_to_sandbox(dst, &env_buffer, n);
        Ok(())
    }

    #[flux::sig(fn(&VmCtx[@cx], SboxPtr, u32, should_follow:bool, HostFd) -> Result<HostPathSafe(should_follow), RuntimeError>)]
    pub fn translate_path(
        &self,
        path: SboxPtr,
        path_len: u32,
        should_follow: bool,
        dirfd: HostFd,
    ) -> Result<HostPathSafe, RuntimeError> {
        if !self.fits_in_lin_mem(path, path_len) {
            return Err(Eoverflow);
        }
        let host_buffer = self.copy_buf_from_sandbox(path, path_len);
        resolve_path(host_buffer, should_follow, dirfd)
        // self.resolve_path(host_buffer)
    }

    pub fn get_homedir(&self) -> RVec<u8> {
        string_to_rvec_u8(&self.homedir)
        // self.homedir.as_bytes().to_vec()
    }

    #[flux::sig(fn(&VmCtx, FitsUsize(2)) -> u16)]
    pub fn read_u16(&self, start: FitsUsize) -> u16 {
        let bytes: [u8; 2] = [self.mem[start], self.mem[start + 1]];
        u16::from_le_bytes(bytes)
    }

    /// read u32 from wasm linear memory
    // Not thrilled about this implementation, but it works
    #[flux::sig(fn(&VmCtx, FitsUsize(4)) -> u32)]
    pub fn read_u32(&self, start: FitsUsize) -> u32 {
        let bytes: [u8; 4] = [
            self.mem[start],
            self.mem[start + 1],
            self.mem[start + 2],
            self.mem[start + 3],
        ];
        u32::from_le_bytes(bytes)
    }

    /// read u64 from wasm linear memory
    // Not thrilled about this implementation, but it works
    // TODO: need to test different implementatiosn for this function
    #[flux::sig(fn(&VmCtx, FitsUsize(8)) -> u64)]
    pub fn read_u64(&self, start: FitsUsize) -> u64 {
        let bytes: [u8; 8] = [
            self.mem[start],
            self.mem[start + 1],
            self.mem[start + 2],
            self.mem[start + 3],
            self.mem[start + 4],
            self.mem[start + 5],
            self.mem[start + 6],
            self.mem[start + 7],
        ];
        u64::from_le_bytes(bytes)
    }

    /// read (u32,u32) from wasm linear memory
    pub fn read_u32_pair(&self, start: usize) -> RuntimeResult<(u32, u32)> {
        if !self.fits_in_lin_mem_usize(start, 8) {
            return Err(Eoverflow);
        }
        let x1 = self.read_u32(start);
        let x2 = self.read_u32(start + 4);
        // Ok(Pair { fst: x1, snd: x2 })
        Ok((x1, x2))
    }

    // TODO @cx is redundant here but due to https://github.com/liquid-rust/flux/issues/158
    #[flux::sig(fn (&mut VmCtx[@cx], FitsUsize(1), v: u8))]
    pub fn write_u8(&mut self, offset: FitsUsize, v: u8) {
        self.mem[offset] = v;
    }

    /// write u16 to wasm linear memory
    // Not thrilled about this implementation, but it works
    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_methods(to_le_bytes)]
    // #[requires(self.fits_in_lin_mem_usize(start, 2, trace))]
    // #[requires(ctx_safe(self))]
    // #[requires(trace_safe(trace, self))]
    // #[ensures(ctx_safe(self))]
    // #[ensures(trace_safe(trace, self))]
    // // #[ensures(effects!(old(trace), trace, effect!(WriteMem, addr, 2) if addr == start as usize))]
    #[flux::sig(fn (&mut VmCtx[@cx], FitsUsize(2), v: u16))]
    pub fn write_u16(&mut self, start: FitsUsize, v: u16) {
        let bytes: [u8; 2] = v.to_le_bytes();
        self.write_u8(start, bytes[0]);
        self.write_u8(start + 1, bytes[1]);
    }

    /// write u32 to wasm linear memory
    // Not thrilled about this implementation, but it works
    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_methods(to_le_bytes)]
    // #[requires(self.fits_in_lin_mem_usize(start, 4, trace))]
    // #[requires(ctx_safe(self))]
    // #[requires(trace_safe(trace, self))]
    // #[ensures(ctx_safe(self))]
    // #[ensures(trace_safe(trace, self))]
    // // #[ensures(effects!(old(trace), trace, effect!(WriteMem, addr, 4) if addr == start as usize))]
    #[flux::sig(fn (&mut VmCtx[@cx], FitsUsize(4), v: u32))]
    pub fn write_u32(&mut self, start: FitsUsize, v: u32) {
        let bytes: [u8; 4] = v.to_le_bytes();
        self.write_u8(start, bytes[0]);
        self.write_u8(start + 1, bytes[1]);
        self.write_u8(start + 2, bytes[2]);
        self.write_u8(start + 3, bytes[3]);
    }

    // TODO: replace with faster raw ptr memread/memwrite
    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_methods(to_le_bytes)]
    // #[requires(self.fits_in_lin_mem_usize(start, 8, trace))]
    // #[requires(ctx_safe(self))]
    // #[requires(trace_safe(trace, self))]
    // #[ensures(ctx_safe(self))]
    // #[ensures(trace_safe(trace, self))]
    // // #[ensures(effects!(old(trace), trace, effect!(WriteMem, addr, 8) if addr == start as usize))]
    #[flux::sig(fn (&mut VmCtx[@cx], FitsUsize(8), v: u64))]
    pub fn write_u64(&mut self, start: FitsUsize, v: u64) {
        let bytes: [u8; 8] = v.to_le_bytes();
        self.write_u8(start, bytes[0]);
        self.write_u8(start + 1, bytes[1]);
        self.write_u8(start + 2, bytes[2]);
        self.write_u8(start + 3, bytes[3]);
        self.write_u8(start + 4, bytes[4]);
        self.write_u8(start + 5, bytes[5]);
        self.write_u8(start + 6, bytes[6]);
        self.write_u8(start + 7, bytes[7]);
    }

    #[flux::qualifiers(MyQ1)]
    #[flux::sig(fn(&VmCtx[@cx], &RVec<WasmIoVec>) -> RVec<NativeIoVecOk(cx.base)>)]
    pub fn translate_iovs(&self, iovs: &RVec<WasmIoVec>) -> RVec<NativeIoVecOk> {
        let mut idx = 0;
        let mut native_iovs = NativeIoVecs::new();
        let iovcnt = iovs.len();
        while idx < iovcnt {
            let iov = iovs[idx];
            let native_iov = self.translate_iov(iov);
            native_iovs.push(native_iov);
            idx += 1;
        }
        native_iovs
    }
}
