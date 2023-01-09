# WaVe: a verifiably secure WebAssembly sandboxing runtime

This repository contains all the code and data necessary for building WaVe and reproducing the results presented in our paper [WaVe: a verifiably secure WebAssembly sandboxing runtime](https://cseweb.ucsd.edu/~dstefan/pubs/johnson:2022:wave.pdf).  
  
## Abstract
The promise of software sandboxing is flexible, fast and portable isolation; capturing the benefits of hardware-based memory protection without requiring operating system involvement. This promise is reified in WebAssembly (Wasm), a popular portable bytecode whose compilers automatically insert
runtime checks to ensure that data and control flow are constrained to a single memory segment. Indeed, modern compiled Wasm implementations have advanced to the point where these checks can themselves be verified, removing the compiler from the trusted computing base. However, the resulting integrity
properties are only valid for code executing strictly inside the Wasm sandbox. Any interactions with the runtime system, which manages sandboxes and exposes the WebAssembly System Interface (WASI) used to access operating system resources, operate outside this contract. The resulting conundrum is how to
maintain Wasm’s strong isolation properties while still allowing such programs to interact with the outside world (i.e., with the file system, the network, etc.). Our paper presents a solution to this problem, via WaVe, a verified secure runtime system that implements WASI. We mechanically verify that interactions with WaVe (including OS side effects) not only maintain Wasm’s memory safety guarantees, but also maintain access isolation for the host OS’s storage and network resources. Finally, in spite of completely removing the runtime from the trusted computing base, we show that WaVe offers performance competitive with existing industrial (yet unsafe) Wasm runtimes.

### Install dependencies
`apt-get install -y curl git unzip build-essential pkg-config libssl-dev cmake ninja-build clang`

Additionally, ensure you have [https://rustup.rs/](Rustup) installed, and install additional Rust dependencies:    
`cargo install --force cbindgen`

### To build and verify:  

```
make bootstrap # Setup build the first time. This will take 15-20 minutes.
make build     # Build WaVe. This should take < 1 minute. 
make verify    # Verify correctness of WaVe. This will take 30-60 minutes.
```

## To run an example application using WaVe
```
cd examples/cat # enter example directory  
make            # build  
make run        # execute cat example  
```
You can examine the makefile to see the exact commands


## Related documentation
Interface for WASI calls: [WASI API](https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md)  

## Reproducing the evaluation
TODO
