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

#[trusted]
#[requires(components_normalized(components))]
#[ensures(old(is_relative(components)) ==> vec_is_relative(&result) )]
#[ensures(old(min_depth(components)) == vec_depth(&result) )]
fn from_components(components: &Vec<Component>) -> Vec<u8> {
    //let p = p.as_path();
    let mut path = PathBuf::new();
    for c in components.iter() {
        path.push(c.as_os_str());
    } 

    from_pathbuf(path)
  }

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



#[trusted]
#[pure]
#[requires(index < vec.len() )]
pub fn lookup_component<'a>(
    vec: &'a Vec<Component<'a>>,
    index: usize,
) -> &'a Component<'a> {
    &vec[index]
}

// If the first component is not the rootdir or a prefix (like Windows C://) its relative
#[requires(c.len() > 0)]
#[pure]
#[trusted]
#[ensures(result == true ==> components_normalized(c) )]
fn is_relative(c: &Vec<Component>) -> bool {
    let start = c[0];
    //let start = lookup_component(c, 0);
    !(matches!(start, Component::Prefix(_) | Component::RootDir))
}



// #[trusted]
// #[pure]
// #[requires(index < vec.len() )]
// pub fn vec_u8_lookup(
//     vec: &Vec<u8>,
//     index: usize,
// ) -> u8 {
//     vec[index]
// }

// #[pure]
// fn pure_sum_h(v: &Vec<u8>, idx: usize, end: usize) -> u8 {
//     if idx >= end || idx >= v.len() {
//         return 0;
//     }
//     pure_sum_h(v, idx + 1, end) + vec_u8_lookup(v, idx)
// }

// #[pure]
// fn pure_sum(v: &Vec<u8>) -> u8 {
//     pure_sum_h(v, 0, v.len())
// }

// fn sum(v: &Vec<u8>) -> u8 {
//     if v.len() <= 0 {
//         return 0;
//     }
//     let mut acc = 0;
//     let mut idx = 0;
//     while idx < v.len() {
//         body_invariant!(idx < v.len());
//         // body_invariant!(acc == pure_sum_h(v, 0, idx));
//         acc += vec_u8_lookup(v, idx);
//         idx += 1;
//     }
//     acc
// } 



// use really big negative number instead of option because the verifier does not like returning options from pure code
// apparently I can make it pure or I can make it untrusted but I cannot do both
#[pure]
#[trusted]
#[requires(components_normalized(components))]
// #[ensures(result == pure_depth(components))]
fn min_depth(components: &Vec<Component>) -> isize {
    let mut curr_depth = 0;
    let mut idx = 0;
    while idx < components.len() {
        // invariant: idx < components.len()
        body_invariant!(curr_depth >= 0);
        body_invariant!(components_normalized(components));
        // pure_depth_h(components, 0)
        match components[idx] {
            Component::Prefix(_) | Component::RootDir => {return DEPTH_ERR;} // hacky, but fine for now
            Component::CurDir => {},
            Component::ParentDir => {curr_depth -= 1;},
            Component::Normal(_) => {curr_depth += 1;},
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

#[cfg(feature = "verify")]
predicate! {
    pub fn components_normalized(components: &Vec<Component>) -> bool {
        forall(|i: usize|
            (i < components.len() ==> (
                matches!(lookup_component(components, i), Component::CurDir | Component::ParentDir | Component::Normal(_))
            ))
        )
    }
}

// #[ensures(result.is_ok() => components_normalized(components))]
// fn normalize_path(components: &Vec<Component>) -> RuntimeResult<Vec<Component>> {
//     let mut idx = 0;
//     while idx < components.len() {
//         if !(matches!(start, Component::Prefix(_) | Component::RootDir)) {

//         }
//     }
//     !(matches!(start, Component::Prefix(_) | Component::RootDir))
// }

#[pure]
#[requires(index < components.len())]
#[requires(components_normalized(components))]
#[trusted]
// checks the correctness of
fn pure_depth_h(components: &Vec<Component>, index: usize) -> isize {
    if index >= components.len() {
        0
    }
    else { 
        let c = lookup_component(components, index);
        pure_depth_h(components, index + 1) + elem_depth(c)
    }
}

#[pure]
#[requires(components_normalized(components))]
fn pure_depth(components: &Vec<Component>) -> isize {
    if components.len() <= 0 {
        return 0;
    }
    // assert!(components_normalized(components));
    pure_depth_h(components, 0)
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

#[ensures(
    match &result {
        Ok(v) => vec_is_relative(&v) && (vec_depth(&v) >= 0),
        _ => true,
    }
)]
pub fn resolve_path(path: Vec<u8>) -> RuntimeResult<Vec<u8>> {
    // let path = expand_path(&path, should_follow)?;
    // let path = to_pathbuf(path);
    let p = to_pathbuf(path);
    let c: Vec<Component> = get_components(&p);
    // let c: Vec<Component> = normalize_path(&c);

    // depth < 0
    if c.len() <= 0 || !is_relative(&c) || min_depth(&c) < 0 {//|| depth(&c).is_some(){
        return Err(RuntimeError::Eacces);
    }

    Ok(from_components(&c))
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


// #[trusted]
// #[requires(!is_symlink(out_path) )]
// #[ensures(!is_symlink(out_path))]
// fn maybe_expand_component(out_path: &mut OwnedComponents, comp: OwnedComponent, num_symlinks: &mut isize) -> Option<OwnedComponents>{
//     out_path.inner.push(comp);
//     let linkpath = read_link_h(out_path)?;
//     out_path.inner.pop(); // pop the component we just added, since it is a symlink
//     *num_symlinks += 1;
//     Some(linkpath)
// }


// #[ensures(x == old(x))]
// fn structural_eq_test(x: &mut Vec<u8>) -> &Vec<u8>{
//     x.push(3);
//     x.pop();
//     x
// }

// compute sum of `Vec<i32>`
// #[pure]
// #[trusted]
// fn vec_sum(v: &Vec<i32>) -> i32 {
//     let mut sum = 0;
//     for x in v.iter() {
//         sum += *x;
//     }
//     sum
// }


// fn sum_test1(){
//     let x: Vec<u8> = Vec::new();
//     let y: Vec<u8> = Vec::new();
//     //assert!(x[..] == y[..]);
// }

// #[ensures(vec_sum(v) == old(vec_sum(v)))]
// fn sum_test2(v: &mut Vec<i32>){
//     v.push(3);
//     v.pop();
//     //assert!(x[..] == y[..]);
// }



// fn structural_eq_test2(){
//     let x: Vec<u8> = Vec::new();
//     let y: Vec<u8> = Vec::new();
//     assert!(x[..] == y[..]);
// }

// returns None if there was some problem (should reject)
// #[trusted]
// #[ensures(result.is_some() && should_follow ==> no_symlinks(get_components(&result.unwrap())  ) )]
// #[ensures(result.is_some() && !should_follow ==> no_symlinks(get_components(&result.unwrap()[..-1])))]


// its an empty path, its not a symlink
#[trusted]
#[ensures(!is_symlink(&result))]
fn fresh_components() -> OwnedComponents {
    OwnedComponents::new()
}


//#[ensures(!is_symlink(out_path))]

fn expand_path(vec: Vec<u8>, should_follow: bool) -> Option<OwnedComponents> {
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
            return None;
        }
        idx += 1;
    }
    Some(out_path)
}


