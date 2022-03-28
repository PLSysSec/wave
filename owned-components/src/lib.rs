use std::path::{Component, Path, PathBuf};
// use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStringExt, OsStrExt};
use std::io::{Error, ErrorKind};
use std::io;
use std::ffi::{CStr, CString, OsStr, OsString};
//use libc;

// Represents an Owned version of a Component<'a>
// Currently only works for *nix (neglects the prefix component present on windows)
#[cfg_attr(not(feature = "verify"), derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub enum OwnedComponent {
    /// The root directory component, appears before anything else.
    ///
    /// It represents a separator that designates that a path starts from root.
    RootDir,

    /// A reference to the current directory, i.e., `.`.
    CurDir,

    /// A reference to the parent directory, i.e., `..`.
    ParentDir,

    /// A normal component, e.g., `a` and `b` in `a/b`.
    ///
    /// This variant is the most common one, it represents references to files
    /// or directories.
    Normal(OsString),
}

impl OwnedComponent {
    pub fn from_borrowed(c: &Component) -> Self {
        match c {
            Component::Prefix(_) => panic!("OwnedComponents do not support prefix"),
            Component::RootDir => OwnedComponent::RootDir,
            Component::CurDir => OwnedComponent::CurDir,
            Component::ParentDir => OwnedComponent::ParentDir,
            Component::Normal(s) => OwnedComponent::Normal(s.to_os_string()),
        }
    }

    // https://doc.rust-lang.org/stable/src/std/path.rs.html#548-556 line 548
    pub fn as_os_string(&self) -> &OsStr {
        match self {
            OwnedComponent::RootDir => OsStr::new("/"),
            OwnedComponent::CurDir => OsStr::new("."),
            OwnedComponent::ParentDir => OsStr::new(".."),
            OwnedComponent::Normal(path) => path,
        }
    }
}

#[cfg_attr(not(feature = "verify"), derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub struct OwnedComponents {
    pub inner: Vec<OwnedComponent>
}

impl OwnedComponents {
    pub fn new() -> Self {
        Self {
            inner: Vec::new()
        }
    }
    // 1 copy:
    pub fn as_pathbuf(&self) -> PathBuf {
        // let s: OsString = self.inner.iter().map(|oc| oc.as_os_string()).collect(); 
        self.inner.iter().map(|oc| oc.as_os_string()).collect()
    }

    // 1 copy
    pub fn parse(p: PathBuf) -> Self {
        let inner: Vec<OwnedComponent> = p.components().map(|c| 
            OwnedComponent::from_borrowed(&c)).collect();
        Self{ inner }
    }

    // Currently requires two copies
    pub fn unparse(self) -> Option<[u8; 4096]> {
        // let s: PathBuf = self.inner.iter().map(|oc| oc.as_os_string()).collect(); 
        let s = self.as_pathbuf().into_os_string();
        // let s = s.into_os_string();
        if s.len() >= 4096 {
            return None;
        }
        let mut out = [0; 4096];
        for (dst, src) in out.iter_mut().zip(s.as_bytes()) { *dst = *src }
        // out.copy_from_slice(s.as_bytes());
        Some(out)
    }

    pub fn lookup(&self, idx: usize) -> OwnedComponent {
        self.inner[idx].clone()
    }

    pub fn push(&mut self, c: OwnedComponent) {
        self.inner.push(c)
    }

    pub fn pop(&mut self) -> Option<OwnedComponent> {
        self.inner.pop()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn prefix(&self, end: usize) -> &OwnedComponents {
        unimplemented!()
    }
}


fn cstr(path: &Path) -> io::Result<CString> {
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

// taken from https://benaaron.dev/rust-docs/src/std/sys/unix/fs.rs.html#1109-1134
// TODO: the principled way to do this is to use our os spec
pub fn readlinkat(fd: usize, p: &Path) -> io::Result<PathBuf> {
    let c_path = cstr(p)?;
    let p = c_path.as_ptr();

    let mut buf = Vec::with_capacity(4096);

    let buf_read = unsafe { libc::readlinkat(fd as i32, p, buf.as_mut_ptr() as *mut _, buf.capacity()) };
    if buf_read == -1 {
        return Err(Error::new(ErrorKind::Other, "path translation readlink failure!"));
    }
    let buf_read = buf_read as usize;

    unsafe {
        buf.set_len(buf_read);
    }

    if buf_read != buf.capacity() {
        buf.shrink_to_fit();
    }
    return Ok(PathBuf::from(OsString::from_vec(buf)));
    

}

