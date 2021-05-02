pub mod wrapper_utils;
use wrapper_utils::*;


type *mut u8 hostptr;
type u32 sandboxptr;


u64 membase = 0x40000000;
u64 memlen =  0x10000000;

fn empty_predicate()->bool{
    return true;
}

fn ptr_to_sandbox(buf: hostptr)->sandboxptr
{
    return buf - membase;
}


fn ptr_from_sandbox(buf: sandboxptr) -> hostptr
{
    return buf + membase;
}


// sandboxptrs 
fn inBounds(ptr: sandboxptr) -> bool{
    return (ptr >= membase) && (ptr <= (membase + memlen));  
}

fn sized_buf_to_sandbox(buf: hostptr, size_v: size) -> samdboxptr
{
    return ptr_to_sandbox(buf);
}

// returns pointer if success, or null if memory violation
hostptr sized_buf_from_sandbox(buf: sandboxptr, size_v: size)
{
    if (inBounds(membase + buf) && inBounds(membase + buf + size_v)){
        return ptr_from_sandbox(buf);
    }
    else{
        return NULL;
    }
}


sandboxptr path_to_sandbox(buf: hostptr)
{
    return sized_buf_to_sandbox(buf, PATH_MAX);
}


hostptr path_from_sandbox(buf: sandboxptr)
{
    return sized_buf_from_sandbox(buf, PATH_MAX);
}


