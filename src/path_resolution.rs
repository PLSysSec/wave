use crate::tcb::path::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use owned_components::{readlinkat, OwnedComponent, OwnedComponents};
use prusti_contracts::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};

const MAXSYMLINKS: isize = 10;

// #[pure]
// #[ensures()]
fn to_pathbuf(v: Vec<u8>) -> PathBuf {
    PathBuf::from(OsString::from_vec(v.clone()))
}

// #[ensures(!is_symlink(out_path))]
#[ensures(
    match &result {
        Ok(v) => forall(|i: usize| (i < v.len() - 1) ==> !is_symlink(v.prefix(i)) ) && 
            (!should_follow || (should_follow && !is_symlink(&v))),
        _ => true,
    }
)]
fn expand_path(vec: Vec<u8>, should_follow: bool, dirfd: HostFd) -> RuntimeResult<OwnedComponents> {
    let p = to_pathbuf(vec);
    let components = get_components(&p);

    let mut out_path = fresh_components();
    let mut num_symlinks = 0;
    let mut idx = 0;

    while idx < components.len() {
        body_invariant!(!is_symlink(&out_path));
        // out_path should never contain symlinks
        body_invariant!(forall(|i: usize| i < out_path.len() ==> !is_symlink(out_path.prefix(i)) ) );
        let comp = components[idx];
        let c = OwnedComponent::from_borrowed(&comp);
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

#[ensures(
    match &result {
        Ok(v) => path_safe(&v, should_follow),
        _ => true,
    }
)]
pub fn resolve_path(path: Vec<u8>, should_follow: bool, dirfd: HostFd) -> RuntimeResult<HostPath> {
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

    match OwnedComponents::unparse(c) {
        Some(result_arr) => Ok(result_arr),
        _ => Err(RuntimeError::Enametoolong),
    }
}

// Recursively expands a symlink (without explicit recursion)
// maintains a queue of path components to process
#[requires(forall(|i: usize| (i < out_path.len()) ==> !is_symlink(out_path.prefix(i)) ))]
#[requires(!is_symlink(out_path) )]
#[ensures(!is_symlink(out_path))]
#[ensures(forall(|i: usize| (i < out_path.len()) ==> !is_symlink(out_path.prefix(i)) ))]
fn expand_symlink(
    out_path: &mut OwnedComponents,
    linkpath_components: OwnedComponents,
    num_symlinks: &mut isize,
    dirfd: HostFd,
) {
    let mut idx = 0;
    while idx < linkpath_components.len() {
        body_invariant!(!is_symlink(out_path));
        // out_path should never contain symlinks
        body_invariant!(forall(|i: usize| i < out_path.len() ==> !is_symlink(out_path.prefix(i))));
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
