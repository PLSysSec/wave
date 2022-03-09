#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use prusti_contracts::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};
use std::fs::read_link;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use owned_components::{OwnedComponents, OwnedComponent};

const DEPTH_ERR: isize = i32::MIN as isize;
const MAXSYMLINKS: isize = 10;


#[trusted]
fn get_components(path: &PathBuf) -> Vec<Component> {
    // let path = PathBuf::from(OsString::from_vec(path));
    path.components().collect()
}

// #[trusted]
// #[requires(components_normalized(components))]
// #[ensures(old(is_relative(components)) ==> vec_is_relative(&result) )]
// #[ensures(old(min_depth(components)) == vec_depth(&result) )]
// fn from_components(components: &Vec<Component>) -> Vec<u8> {
//     //let p = p.as_path();
//     let mut path = PathBuf::new();
//     for c in components.iter() {
//         path.push(c.as_os_str());
//     } 

//     from_pathbuf(path)
//   }

// #[pure]
// #[ensures()]
fn to_pathbuf(v: Vec<u8>) -> PathBuf {
    PathBuf::from(OsString::from_vec(v.clone()))
}

// TODO: remove this clone: its only there because prusti complains
fn from_pathbuf(p: PathBuf) -> Vec<u8> {
  // OsString are not guaranteed to be null terminated, but we want it to be null-terminated :)
  let mut v = p.clone().into_os_string().into_vec();
  v.push(0);
  v
}



#[requires(idx < 4)]
#[pure]
#[trusted]
pub fn addr_matches_netlist_entry(netlist: &Netlist, addr: u32, port: u32, idx: usize) -> bool {
    addr == netlist[idx].addr && port == netlist[idx].port
}



// #[trusted]
// #[pure]
// #[requires(index < vec.len() )]
// pub fn lookup_component<'a>(
//     vec: &'a Vec<Component<'a>>,
//     index: usize,
// ) -> &'a Component<'a> {
//     &vec[index]
// }

// If the first component is not the rootdir or a prefix (like Windows C://) its relative
#[requires(c.len() > 0)]
#[pure]
#[trusted]
// #[ensures(result == true ==> components_normalized(c) )]
fn is_relative(c: &OwnedComponents) -> bool {
    let start = c.lookup(0);
    //let start = lookup_component(c, 0);
    !(matches!(start, OwnedComponent::RootDir))
}


// use really big negative number instead of option because the verifier does not like returning options from pure code
// apparently I can make it pure or I can make it untrusted but I cannot do both
#[pure]
#[trusted]
// #[requires(components_normalized(components))]
// #[ensures(result == pure_depth(components))]
fn min_depth(components: &OwnedComponents) -> isize {
    let mut curr_depth = 0;
    let mut idx = 0;
    while idx < components.len() {
        // invariant: idx < components.len()
        body_invariant!(curr_depth >= 0);
        // body_invariant!(components_normalized(components));
        // pure_depth_h(components, 0)
        match components.lookup(idx) {
            OwnedComponent::RootDir => {return DEPTH_ERR;} // hacky, but fine for now
            OwnedComponent::CurDir => {},
            OwnedComponent::ParentDir => {curr_depth -= 1;},
            OwnedComponent::Normal(_) => {curr_depth += 1;},
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

#[pure]
#[requires(matches!(component, Component::CurDir | Component::ParentDir | Component::Normal(_)))]
fn elem_depth(component: &Component) -> isize {
    match component {
        Component::CurDir => 0,
        Component::ParentDir => -1,
        Component::Normal(_) => 1,
        _ => panic!(),
    }
}

// bodyless viper program
#[pure]
#[trusted]
fn vec_is_relative(v: &Vec<u8>) -> bool {
    panic!()
}

// bodyless viper program
#[pure]
#[trusted]
fn vec_depth(components: &Vec<u8>) -> isize {
    panic!()
}

// #[ensures(
//     match &result {
//         Ok(v) => vec_is_relative(&v) && (vec_depth(&v) >= 0),
//         _ => true,
//     }
// )]
pub fn resolve_path(path: Vec<u8>) -> RuntimeResult<Vec<u8>> {
    // let p = to_pathbuf(path);
    // let c: Vec<Component> = get_components(&p);

    // TODO: replace true with SHOULD_FOLLOW 
    let c = expand_path(path, true)?;

    // depth < 0
    if c.len() <= 0 || !is_relative(&c) || min_depth(&c) < 0 {//|| depth(&c).is_some(){
        return Err(RuntimeError::Eacces);
    }

    Ok(OwnedComponents::unparse(c))
    //Some(())
}








// https://man7.org/linux/man-pages/man7/path_resolution.7.html
// linux fails on nonexistant paths so nonexistant_path/.. will fail
// accordingly, we do not elimintate nonexistant paths


// Recursively expands a symlink (without explicit recursion)
// maintains a queue of path components to process
// #[trusted]
#[requires(!is_symlink(out_path) )]
#[ensures(!is_symlink(out_path))]
fn expand_symlink(out_path: &mut OwnedComponents, linkpath_components: OwnedComponents, num_symlinks: &mut isize){
    // let components = get_components(&linkpath);
    let mut idx = 0;
    //for c in components.iter() {
    while idx < linkpath_components.len() {
        body_invariant!(!is_symlink(out_path));
        if *num_symlinks >= MAXSYMLINKS {
            return;
        }
        //let c = linkpath_components.inner[idx].clone();
        let c = linkpath_components.lookup(idx);
        let maybe_linkpath = maybe_expand_component(out_path, c, num_symlinks);
        if let Some(linkpath) = maybe_linkpath {
            // Fine, I'll do the recursion, whatever
            expand_symlink(out_path, linkpath, num_symlinks);
        }
        idx += 1;
    }
}

// bodyless viper program
#[pure]
#[trusted]
fn is_symlink(components: &OwnedComponents) -> bool {
    panic!()
}

#[trusted]
#[ensures(result.is_none() ==> old(!is_symlink(out_path)) )]
fn read_link_h(out_path: &OwnedComponents) -> Option<OwnedComponents> {
    // let p = out_path.inner.iter().collect::<PathBuf>();
    read_link(out_path.as_path()).ok().map(|p| OwnedComponents::parse(p))
    // match comp {
    //     Component::Normal(p) => read_link(&out_path.join(p)).ok(),
    //     other => None,
    // }
}

// Looks at a single component of a path:
// if it is a symlink, return the linkpath.
// else, we just append the value to out_path
#[trusted]
#[requires(!is_symlink(out_path) )]
#[ensures(!is_symlink(out_path))]
fn maybe_expand_component(out_path: &mut OwnedComponents, comp: OwnedComponent, num_symlinks: &mut isize) -> Option<OwnedComponents>{
    out_path.inner.push(comp);
    if let Some(linkpath) = read_link_h(out_path) {
        out_path.inner.pop(); // pop the component we just added, since it is a symlink
        // assert!(!is_symlink(out_path));
        *num_symlinks += 1;
        return Some(linkpath);
    }
    assert!(!is_symlink(out_path));
    return None;
    
}

// its an empty path, its not a symlink
#[trusted]
#[ensures(!is_symlink(&result))]
fn fresh_components() -> OwnedComponents {
    OwnedComponents::new()
}


//#[ensures(!is_symlink(out_path))]

fn expand_path(vec: Vec<u8>, should_follow: bool) -> RuntimeResult<OwnedComponents> {
    let p = to_pathbuf(vec);
    let components = get_components(&p);

    // let mut out_path = OwnedComponents::new();
    let mut out_path = fresh_components();
    let mut num_symlinks = 0;
    let mut idx = 0;
    assert!(!is_symlink(&out_path));


    while idx < components.len() {
        body_invariant!(!is_symlink(&out_path));
        let comp = components[idx];
        let c = OwnedComponent::from_borrowed(&comp);
        // if this is the last element, and we are NO_FOLLOW, then don't expand
        if !should_follow && idx + 1 == components.len(){
            out_path.push(c);
            break;
        }
        // if comp is a symlink, return path + update num_symlinks
        // if not, just extend out_path
        let maybe_linkpath = maybe_expand_component(&mut out_path, c, &mut num_symlinks);
        if let Some(linkpath) = maybe_linkpath {
            expand_symlink(&mut out_path, linkpath, &mut num_symlinks);
        }
        if num_symlinks >= MAXSYMLINKS {
            return Err(RuntimeError::Eloop);
        }
        idx += 1;
    }
    Ok(out_path)
}


