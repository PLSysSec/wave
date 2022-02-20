#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use prusti_contracts::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};

#[trusted]
fn get_components(path: &PathBuf) -> Vec<Component> {
    path.components().collect()
}

impl VmCtx {
    /// Check whether a path is in the home directory.
    /// If it is, return it as an absolute path, if it isn't, return error
    // TODO: verify and make untrusted
    // #[with_ghost_var(trace: &mut Trace)]
    #[trusted]
    // #[requires(trace_safe(trace, self) && ctx_safe(self))]
    // #[ensures(trace_safe(trace, self) && ctx_safe(self))]
    pub fn resolve_path(&self, in_path: Vec<u8>) -> RuntimeResult<SandboxedPath> {
        let path = PathBuf::from(OsString::from_vec(in_path));
        // println!("resolve_path: path = {:?}", path);
        let safe_path = PathBuf::from(self.homedir.clone()).join(normalize_path(&path));
        // println!("safe_path: safe_path = {:?}", safe_path);
        let path_str = safe_path.into_os_string();
        // println!("path_str = {:?}, into_string = ", path_str, path_str.into_string());
        if let Ok(s) = path_str.into_string() {
            // println!("Checking prefix of s = {:?} as_bytes = {:?}", s, s.clone().into_bytes());
            if s.starts_with(&self.homedir) {
                let mut bytepath = s.into_bytes();
                bytepath.push(0); // push null
                return Ok(SandboxedPath::from(bytepath));
            }
        }
        Err(RuntimeError::Eacces)
    }
}

#[requires(idx < 4)]
#[pure]
#[trusted]
pub fn addr_matches_netlist_entry(netlist: &Netlist, addr: u32, port: u32, idx: usize) -> bool {
    addr == netlist[idx].addr && port == netlist[idx].port
}

/// Convert relative path to absolute path
/// Used to check that that paths are sandboxed
// TODO: verify this
// Prusti does not like this function at all
// #[trusted]
pub fn normalize_path(path: &PathBuf) -> PathBuf {
    // let components = path.components();
    // let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
    //     components.next();
    //     PathBuf::from(c.as_os_str())
    // } else {
    //     PathBuf::new()
    // };
    let mut ret = PathBuf::new();
    let components = get_components(path);
    let mut idx = 0;
    while idx < components.len() {
        // for component in components.iter() {
        let component = components[idx];
        match component {
            Component::Prefix(..) => {} /*unreachable!(),*/
            Component::RootDir => {
                //ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                // ret.pop();
            }
            Component::Normal(_) => {
                //ret.push(c);
            }
        }
    }
    ret
}

/*


// https://man7.org/linux/man-pages/man7/path_resolution.7.html
// linux fails on nonexistant paths so nonexistant_path/.. will fail
// accordingly, we do not elimintate nonexistant paths


// Recursively expands a symlink (without explicit recursion)
// maintains a queue of path components to process
#[trusted]
fn expand_symlink(out_path: &mut PathBuf, linkpath: PathBuf, num_symlinks: &mut isize){
    for c in linkpath.components(){
        if *num_symlinks >= MAXSYMLINKS {
            return;
        }
        let maybe_linkpath = maybe_expand_component(out_path, &c, num_symlinks);
        if let Some(linkpath) = maybe_linkpath {
            // Fine, I'll do the recursion, whatever
            expand_symlink(out_path, linkpath, num_symlinks);
        }
    }
}

/// Looks at a single component of a path:
/// if it is a symlink, return the linkpath.
/// else, we just append the value to out_path
#[trusted]
//#[requires()] requires out_path.depth() > 0
fn maybe_expand_component(out_path: &mut PathBuf, comp: &Component, num_symlinks: &mut isize) -> Option<PathBuf>{
    if let Component::Normal(p) = comp {
        if let Ok(linkpath) = read_link(&out_path.join(p)){
            *num_symlinks += 1;
            return Some(linkpath);
        }
    }
    out_path.push(comp);
    None


}

// returns None if there was some problem (should reject)
#[trusted]
fn expand_path(vec: &Vec<u8>, should_follow: bool) -> Option<PathBuf> {
    let path = PathBuf::from(OsStr::from_bytes(vec));
    let components: Vec<Component> = path.components().collect();
    let mut out_path = PathBuf::new();

    let mut num_symlinks = 0;
    let mut idx = 0;

    for comp in components.iter(){
        idx += 1;
        // if this is the last element, and we are NO_FOLLOW, then don't expand
        if !should_follow && idx == components.len(){
            out_path.push(comp);
            break;
        }
        let maybe_linkpath = maybe_expand_component(&mut out_path, comp, &mut num_symlinks);
        if let Some(linkpath) = maybe_linkpath {
            expand_symlink(&mut out_path, linkpath, &mut num_symlinks);
        }
        if num_symlinks >= MAXSYMLINKS {
            return None;
        }
    }

    Some(out_path)
}

// If the first component is not the rootdir or a prefix (like Windows C://) its relative
#[requires(c.len() > 0)]
// #[pure]
// #[trusted]
fn is_relative(c: &Vec<Component>) -> bool {
    let start = c[0];
    !(matches!(start, Component::Prefix(_) | Component::RootDir))
}


// Checks a path. We let the *at calls do the actual path extension
// If the path is absolute ==> return false
// if the absolute ==> return depth > 0
// #[trusted]
fn check_path(path: Vec<u8>, should_follow: bool) -> Option<()> {
    let path = expand_path(&path, should_follow)?;
    let c: Vec<Component> = get_components(&path);

    if c.len() <= 0 || !is_relative(&c) || depth(c) <= 0{
        return None;
    }
    Some(())
}


// use really big negative number instead of option because the verifier does not like returning options from pure code
// apparently I can make it pure or I can make it untrusted but I cannot do both
#[pure]
#[trusted]
fn depth(components: Vec<Component>) -> isize {
    let mut curr_depth = 0;
    let mut idx = 0;
    while idx < components.len() {
        match components[idx] {
            Component::Prefix(_) | Component::RootDir => {return DEPTH_ERR;} // not allowed!
            Component::CurDir => {},
            Component::ParentDir => {curr_depth -= 1;},
            Component::Normal(_) => {curr_depth += 1;},
        };
        // if curr_depth ever dips below 0, it is illegal
        // this prevents paths like ../other_sandbox_home
        if curr_depth < 0 {
            return DEPTH_ERR;
        }
        idx += 1;
    }
    curr_depth
}


*/
