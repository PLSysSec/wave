#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use prusti_contracts::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};

impl VmCtx {
    /// Check whether a path is in the home directory.
    /// If it is, return it as an absolute path, if it isn't, return error
    // TODO: verify and make untrusted
    // #[with_ghost_var(trace: &mut Trace)]
    #[trusted]
    // #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    // #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn resolve_path(&self, in_path: Vec<u8>) -> RuntimeResult<SandboxedPath> {
        let path = PathBuf::from(OsString::from_vec(in_path));
        //println!("resolve_path: path = {:?}", path);
        let safe_path = PathBuf::from(self.homedir.clone()).join(normalize_path(&path));
        //println!("safe_path: safe_path = {:?}", safe_path);
        let path_str = safe_path.into_os_string();
        //println!("path_str = {:?}, into_string = ", path_str, path_str.into_string());
        if let Ok(s) = path_str.into_string() {
            //println!("Checking prefix of s = {:?}", s);
            if s.starts_with(&self.homedir) {
                return Ok(SandboxedPath::from(s.into_bytes()));
            }
        }
        Err(RuntimeError::Eacces)
    }
}

/// Convert relative path to absolute path
/// Used to check that that paths are sandboxed
// TODO: verify this
// Prusti does not like this function at all
#[trusted]
pub fn normalize_path(path: &PathBuf) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
