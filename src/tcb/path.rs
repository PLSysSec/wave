use std::path::PathBuf;

use owned_components::{readlinkat, OwnedComponent, OwnedComponents};

use crate::{
    rvec::RVec,
    types::{HostFd, Netlist},
};

#[flux::constant]
const DEPTH_ERR: isize = i32::MIN as isize;

#[allow(dead_code)]
#[flux::opaque]
#[flux::refined_by(depth:int, is_relative:bool, non_symlink:bool, non_symlink_prefixes:bool)]
pub struct HostPath {
    pub inner: [u8; crate::types::PATH_MAX],
}

impl HostPath {
    #[flux::trusted]
    pub fn as_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }
}

#[flux::opaque]
#[flux::refined_by(size:int, ns_prefix:int, depth:int, is_relative:bool)]
pub struct FOwnedComponents {
    inner: OwnedComponents,
}

#[flux::alias(type HostPathOc(oc) = HostPath{ v: v.depth == oc.depth && v.is_relative == oc.is_relative
                                              && v.non_symlink == (oc.size == oc.ns_prefix)
                                              && v.non_symlink_prefixes == (oc.size - 1 <= oc.ns_prefix) })]
pub type _HostPathOc = HostPath;

#[flux::alias(type NoSymLinks = FOwnedComponents{v: v.size == v.ns_prefix})]
pub type NoSymLinks_ = FOwnedComponents;

#[flux::alias(type LastSymLink(b) = FOwnedComponents{v: v.size - 1 <= v.ns_prefix && (b => v.size == v.ns_prefix)})]
pub type LastSymLink_ = FOwnedComponents;

impl FOwnedComponents {
    #[flux::trusted]
    #[flux::sig(fn (&FOwnedComponents[@self]) -> usize[self.size])]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[flux::trusted]
    #[flux::sig(fn (&FOwnedComponents[@self], idx:usize{0 <= idx && idx < self.size}) -> OwnedComponent)]
    pub fn lookup(&self, idx: usize) -> OwnedComponent {
        self.inner.lookup(idx)
    }

    #[flux::trusted]
    #[flux::sig(fn() -> FOwnedComponents[0, 0, 0, false])]
    pub fn new() -> FOwnedComponents {
        FOwnedComponents {
            inner: OwnedComponents::new(),
        }
    }

    #[flux::trusted]
    #[flux::sig(fn (self: &strg FOwnedComponents[@oc], OwnedComponent) -> ()
                ensures self: FOwnedComponents{v: v.size == oc.size + 1 && v.ns_prefix == oc.ns_prefix} )]
    pub fn push(&mut self, value: OwnedComponent) {
        self.inner.push(value);
    }

    #[flux::trusted]
    #[flux::sig(fn (oc:FOwnedComponents) -> Option<HostPathOc[oc]>)]
    pub fn unparse(self) -> Option<HostPath> {
        let inner = self.inner.unparse()?;
        Some(HostPath { inner })
    }
}

#[flux::trusted]
#[flux::sig(fn(&PathBuf) -> RVec<OwnedComponent>)]
pub fn get_components(path: &PathBuf) -> RVec<OwnedComponent> {
    let mut components = RVec::new();
    for c in path.components() {
        components.push(OwnedComponent::from_borrowed(&c));
    }
    components
}

// #[derive(Clone, Copy, PartialEq, Eq)]
pub struct NetEndpoint {
    pub protocol: WasiProto,
    pub addr: u32,
    pub port: u32,
}

// #[derive(Clone, Copy, PartialEq, Eq)]

pub enum WasiProto {
    Unknown,
    Tcp,
    Udp,
}

// If the first component is not the rootdir or a prefix (like Windows C://) its relative
#[flux::trusted]
#[flux::sig(fn(&{FOwnedComponents[@oc] : oc.size > 0}) -> bool[oc.is_relative])]
pub fn is_relative(c: &FOwnedComponents) -> bool {
    let start = c.inner.lookup(0);
    !(matches!(start, OwnedComponent::RootDir))
}

// use really big negative number instead of option because the verifier does not like returning options from pure code
// apparently I can make it pure or I can make it untrusted but I cannot do both

#[flux::trusted]
#[flux::sig(fn (&FOwnedComponents[@oc]) -> isize[oc.depth])]
pub fn min_depth(components: &FOwnedComponents) -> isize {
    let mut curr_depth = 0;
    let mut idx = 0;
    while idx < components.len() {
        match components.inner.lookup(idx) {
            OwnedComponent::RootDir => {
                return DEPTH_ERR;
            } // hacky, but fine for now
            OwnedComponent::CurDir => {}
            OwnedComponent::ParentDir => {
                curr_depth -= 1;
            }
            OwnedComponent::Normal(_) => {
                curr_depth += 1;
            }
        };
        // if curr_depth ever dips below 0, it is illegal
        // this prevents paths like ../other_sandbox_home
        if curr_depth < 0 {
            return curr_depth;
        }
        idx += 1;
    }
    curr_depth
}

#[flux::trusted]
#[flux::sig(fn (HostFd, &FOwnedComponents[@oc]) -> Option<{FOwnedComponents: oc.ns_prefix == oc.size}>)]
fn read_linkat_h(dirfd: HostFd, out_path: &FOwnedComponents) -> Option<FOwnedComponents> {
    let inner = readlinkat(dirfd.to_raw(), &out_path.inner.as_pathbuf())
        .ok()
        .map(|p| OwnedComponents::parse(p))?;
    Some(FOwnedComponents { inner })
}

// Looks at a single component of a path:
// if it is a symlink, return the linkpath.
// else, we just append the value to out_path
#[flux::trusted]
#[flux::sig(fn (HostFd, &mut NoSymLinks, OwnedComponent, &mut isize) -> Option<FOwnedComponents>)]
pub fn maybe_expand_component(
    dirfd: HostFd,
    out_path: &mut FOwnedComponents,
    comp: OwnedComponent,
    num_symlinks: &mut isize,
) -> Option<FOwnedComponents> {
    out_path.inner.push(comp);
    if let Some(linkpath) = read_linkat_h(dirfd, out_path) {
        out_path.inner.pop(); // pop the component we just added, since it is a symlink
        *num_symlinks += 1;
        return Some(linkpath);
    }
    return None;
}

#[flux::trusted]
#[flux::sig(fn () -> FOwnedComponents[0, 0, 0, false])]
pub fn fresh_components() -> FOwnedComponents {
    FOwnedComponents {
        inner: OwnedComponents::new(),
    }
}

#[flux::alias(type CountSafe(ptr) = usize{cnt: fits_in_lin_mem(ptr, cnt) && cnt < LINEAR_MEM_SIZE })]
pub type _CountSafe = usize;

#[flux::alias(type HostPathSafe(b) = HostPath{v: v.depth >= 0 && v.is_relative && (b => v.non_symlink) && v.non_symlink_prefixes})]
pub type _HostPathSafe = HostPath;

// FLUX-TODO2: (alias) gross, should allow using aliases to define aliases i.e. the below should be
// HostPathFlags(flags) = HostPathSafe[!flag_set(flags, AT_SYMLINK_NOFOLLOW)]
// #[flux::alias(type HostPathFlags(flags) = HostPathSafe[!flag_set(flags, AT_SYMLINK_NOFOLLOW)])]
pub type _HostPathFlags = HostPath;
