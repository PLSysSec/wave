#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use prusti_contracts::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};
//use std::fs::read_link;
use owned_components::{readlinkat, OwnedComponent, OwnedComponents};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::str;

const DEPTH_ERR: isize = i32::MIN as isize;

// Uninterpreted functions

#[pure]
#[trusted]
pub fn arr_is_relative(v: &HostPath) -> bool {
    panic!()
}

#[pure]
#[trusted]
pub fn arr_depth(components: &HostPath) -> isize {
    panic!()
}

#[pure]
#[trusted]
pub fn is_symlink(components: &OwnedComponents) -> bool {
    panic!()
}

#[pure]
#[trusted]
pub fn arr_is_symlink(components: &HostPath) -> bool {
    panic!()
}

#[pure]
#[trusted]
pub fn arr_has_no_symlink_prefixes(components: &HostPath) -> bool {
    panic!()
}

#[extern_spec]
impl OwnedComponents {
    #[pure]
    fn len(&self) -> usize;

    pub fn new() -> OwnedComponents;

    #[ensures(self.len() == old(self.len()) + 1)]
    #[ensures(forall(|i: usize| 
        (i < self.len() - 1) ==> 
            (old(!is_symlink(self.prefix(i))) ==>
                !is_symlink(self.prefix(i)) )))]
    pub fn push(&mut self, value: OwnedComponent);

    #[ensures(
        match &result {
            Some(v) => old(is_relative(&self)) ==> arr_is_relative(&v) && 
            old(min_depth(&self)) == arr_depth(&v) && 
            old(is_symlink(&self)) == arr_is_symlink(&v) &&
            old(has_no_symlink_prefixes(&self))  == arr_has_no_symlink_prefixes(&v),
            // forall(|i: usize| 
            //     (i < self.len()) ==> 
            //         (old(is_symlink(self.prefix(i))) == arr_is_symlink(v.prefix(i)) ))
            _ => true,
        }
    )]
    pub fn unparse(self) -> Option<[u8; 4096]>;

    #[pure]
    pub fn prefix(&self, end: usize) -> &OwnedComponents;
}

#[trusted]
pub fn get_components(path: &PathBuf) -> Vec<Component> {
    path.components().collect()
}

#[requires(idx < 4)]
#[pure]
#[trusted]
pub fn addr_matches_netlist_entry(netlist: &Netlist, addr: u32, port: u32, idx: usize) -> bool {
    addr == netlist[idx].addr && port == netlist[idx].port
}

// If the first component is not the rootdir or a prefix (like Windows C://) its relative
#[requires(c.len() > 0)]
#[pure]
#[trusted]
pub fn is_relative(c: &OwnedComponents) -> bool {
    let start = c.lookup(0);
    !(matches!(start, OwnedComponent::RootDir))
}

// use really big negative number instead of option because the verifier does not like returning options from pure code
// apparently I can make it pure or I can make it untrusted but I cannot do both
#[pure]
#[trusted]
pub fn min_depth(components: &OwnedComponents) -> isize {
    let mut curr_depth = 0;
    let mut idx = 0;
    while idx < components.len() {
        body_invariant!(curr_depth >= 0);
        match components.lookup(idx) {
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

#[trusted]
#[ensures(result.is_none() ==> old(!is_symlink(out_path)) )]
fn read_linkat_h(dirfd: HostFd, out_path: &OwnedComponents) -> Option<OwnedComponents> {
    readlinkat(dirfd.to_raw(), &out_path.as_pathbuf())
        .ok()
        .map(|p| OwnedComponents::parse(p))
}

// Looks at a single component of a path:
// if it is a symlink, return the linkpath.
// else, we just append the value to out_path
#[trusted]
#[requires(!is_symlink(out_path) )]
// require that out_path does not contain any symlinks going in
#[requires(forall(|i: usize| (i < out_path.len()) ==> !is_symlink(out_path.prefix(i)) ))]
#[ensures(!is_symlink(out_path))]
// ensures that out_path contains no symlinks on exit
#[ensures(forall(|i: usize| (i < out_path.len()) ==> !is_symlink(out_path.prefix(i)) ))]
pub fn maybe_expand_component(
    dirfd: HostFd,
    out_path: &mut OwnedComponents,
    comp: OwnedComponent,
    num_symlinks: &mut isize,
) -> Option<OwnedComponents> {
    out_path.inner.push(comp);
    if let Some(linkpath) = read_linkat_h(dirfd, out_path) {
        out_path.inner.pop(); // pop the component we just added, since it is a symlink
        *num_symlinks += 1;
        return Some(linkpath);
    }
    return None;
}

// its an empty path, its not a symlink
#[trusted]
#[ensures(result.len() == 0)]
#[ensures(!is_symlink(&result))]
#[ensures(forall(|i: usize| (i < result.len()) ==> !is_symlink(result.prefix(i)) ))] // we should be able to solve this by knowing that length = 0
pub fn fresh_components() -> OwnedComponents {
    OwnedComponents::new()
}

#[cfg(feature = "verify")]
predicate! {
    pub fn has_no_symlink_prefixes(v: &OwnedComponents) -> bool {
        forall(|i: usize| (i < v.len() - 1) ==> !is_symlink(v.prefix(i)))
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn path_safe(v: &HostPath, should_follow: bool) -> bool {
        arr_is_relative(&v) &&
        (arr_depth(&v) >= 0) &&
        (should_follow ==> !arr_is_symlink(&v)) &&
        arr_has_no_symlink_prefixes(&v)
    }
}
