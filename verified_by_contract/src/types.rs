use prusti_contracts::*;

pub const MAX_SBOX_FDS: i32 = 8;
pub const MAX_HOST_FDS: isize = 1024;
pub const PATH_MAX: usize = 1024;

// #define SFI_SAFE(ctx) (true) //This is handled by the builtin memory safety checker

// #define FD_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this 
// #define PATH_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this 
// #define RESOURCE_SAFE(ctx) FD_SAFE(ctx) && PATH_SAFE(ctx) 

// #define SAFE(ctx) VALID_CTX(ctx) && SFI_SAFE(ctx) && RESOURCE_SAFE(ctx) 

//typedef char* hostptr;
pub type HostPtr = usize;
pub type SboxPtr = u32;

pub type HostFd = isize;
pub type SboxFd = i32;



pub struct VmCtx {
    pub membase: usize,
    pub memlen: usize,
    pub fd_sbox_to_host: [HostFd; MAX_HOST_FDS as usize], 
    pub counter: i32,
}