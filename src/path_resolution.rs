use crate::rvec::RVec;
use crate::tcb::path::*;
use crate::types::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

const MAXSYMLINKS: isize = 10;

fn to_pathbuf(v: RVec<u8>) -> PathBuf {
    PathBuf::from(OsString::from_vec(v.to_vec()))
}

#[flux::sig(fn (RVec<u8>, should_follow:bool, HostFd) -> Result<LastSymLink[should_follow], RuntimeError>)]
fn expand_path(
    vec: RVec<u8>,
    should_follow: bool,
    dirfd: HostFd,
) -> Result<FOwnedComponents, RuntimeError> {
    let p = to_pathbuf(vec);
    let components = get_components(&p);

    let mut out_path = fresh_components();
    let mut num_symlinks = 0;

    let mut idx = 0;
    while idx < components.len() {
        let c = components[idx].clone();
        // if this is the last element, and we are NO_FOLLOW, then don't expand
        if !should_follow && idx + 1 == components.len() {
            out_path.push(c);
            break;
        }
        // if comp is a symlink, return path + update num_symlinks
        // if not, just extend out_path
        let maybe_linkpath = maybe_expand_component(dirfd, &mut out_path, c, &mut num_symlinks);

        if let Some(linkpath) = maybe_linkpath {
            expand_symlink(&mut out_path, linkpath, &mut num_symlinks, dirfd);
        }
        if num_symlinks >= MAXSYMLINKS {
            return Err(RuntimeError::Eloop);
        }
        idx += 1;
    }
    //assert!(!should_follow || (should_follow && !is_symlink(&out_path)));
    Ok(out_path)
}

#[flux::sig(fn(RVec<u8>, should_follow:bool, HostFd) -> Result<HostPathSafe[should_follow], RuntimeError>)]
pub fn resolve_path(
    path: RVec<u8>,
    should_follow: bool,
    dirfd: HostFd,
) -> Result<HostPath, RuntimeError> {
    // TODO: use ? when that works properly in Prusti
    let c = expand_path(path, should_follow, dirfd);

    let c = match c {
        Ok(oc) => oc,
        Err(e) => {
            return Err(e);
        }
    };

    if c.len() <= 0 || !is_relative(&c) || min_depth(&c) < 0 {
        return Err(RuntimeError::Enotcapable);
    }

    match FOwnedComponents::unparse(c) {
        Some(result_arr) => Ok(result_arr),
        _ => Err(RuntimeError::Enametoolong),
    }
}

// Recursively expands a symlink (without explicit recursion)
// maintains a queue of path components to process
#[flux::sig(fn(out_path: &mut NoSymLinks, linkpath: FOwnedComponents, num_symlinks: &mut isize, HostFd))]
fn expand_symlink(
    out_path: &mut FOwnedComponents,
    linkpath_components: FOwnedComponents,
    num_symlinks: &mut isize,
    dirfd: HostFd,
) {
    let mut idx = 0;
    while idx < linkpath_components.len() {
        // out_path should never contain symlinks
        if *num_symlinks >= MAXSYMLINKS {
            return;
        }
        let c = linkpath_components.lookup(idx);
        let maybe_linkpath = maybe_expand_component(dirfd, out_path, c, num_symlinks);
        if let Some(linkpath) = maybe_linkpath {
            expand_symlink(out_path, linkpath, num_symlinks, dirfd);
        }
        idx += 1;
    }
}
