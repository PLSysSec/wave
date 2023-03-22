#![feature(register_tool)]
#![register_tool(flux)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![allow(unused_comparisons)]
#![feature(custom_inner_attributes)]
#![flux::qualifier(local, MyQ1(x: int, y: int, a: int) { x + y <= a + LINEAR_MEM_SIZE } )]
// #![flux::def(fits_in_lin_mem(buf: int, cnt: int) -> bool { 0 <= buf && 0 <= cnt && buf <= buf + cnt && buf + cnt < LINEAR_MEM_SIZE } )]
#![flux::def(fits_in_lin_mem_base(base:int, addr: int, count: int) -> bool {
               0 <= count &&
               addr <= addr + count &&
               base <= addr &&
               addr + count < base + LINEAR_MEM_SIZE
   })]
#![flux::def(nth_bit_set(bv: int, n: int) -> bool)]
#![flux::def(flag_set(bv: int, flag: int) -> bool)]
#![flux::def(flag_not_set(bv: int, flag: int) -> bool { bv == 0 || !flag_set(bv, flag) })]
#![flux::def(fits_in_lin_mem(buf: int, count: int) -> bool { fits_in_lin_mem_base(0, buf, count) })]
#![flux::def(addr_matches_netlist_entry(net: int, addr: int, port: int, n: int) -> bool)]
#![flux::def(addr_in_netlist(net: int, addr: int, port: int) -> bool {
      addr_matches_netlist_entry(net, addr, port, 0)
   || addr_matches_netlist_entry(net, addr, port, 1)
   || addr_matches_netlist_entry(net, addr, port, 2)
   || addr_matches_netlist_entry(net, addr, port, 3)
  })]
#![flux::def(WriteMem(base:int, addr:int, count: int) -> bool { fits_in_lin_mem_base(base, addr, count) })]
#![flux::def(ReadMem(base:int, addr:int, count: int) -> bool { fits_in_lin_mem_base(base, addr, count) })]
#![flux::def(ReadMem(base:int, addr:int, count: int) -> bool { fits_in_lin_mem_base(base, addr, count) })]
#![flux::def(Shutdown() -> bool { true })]
#![flux::def(FdAccess() -> bool { true })]
#![flux::def(PathAccessAt(dirfd: int, homedir_host_fd: int) -> bool { dirfd == homedir_host_fd })]
#![flux::def(SockCreation(domain: int, ty: int) -> bool { domain == AF_INET && (ty == SOCK_STREAM || ty == SOCK_DGRAM) })]
#![flux::def(NetAccess(net:int, addr:int, port:int) -> bool { addr_in_netlist(net, addr, port) })]
// #![flux::def(okWriteMem(buf:BSlice, count: int) -> bool { fits_in_lin_mem_base(buf.base, buf.addr, count) })]

mod fdmap;
mod iov;
pub mod lucet_frontend;
mod os;
mod path_resolution;
mod poll;
pub mod runtime;
pub mod rvec;
pub mod stats;
pub mod tcb;
mod tests;
pub mod types;
pub mod verifier_interface;
pub mod wasm2c_frontend;
mod wrappers;
mod writeback;
