use std::process::Command;
use regex::Regex;
use std::path::Path;

// execute an example using our example/make infrastructure
// returns stdout
fn run_and_capture(example_path: &str) -> String {
    let output = Command::new("make")
    .arg("-s") // silent
    .arg("-C") // change directory to example dir
    .arg(example_path)
    .arg("run")
    .output()
    .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

    match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}


#[test]
fn example_cat() {
    let s = run_and_capture("examples/cat");
    assert!(s == "This is the contents of the file!\n");
}

// #[test]
// fn example_client() {
//     let s = run_and_capture("examples/client");
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_client_send_recv() {
//     let s = run_and_capture("examples/client_send_recv");
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_clock() {
//     let s = run_and_capture("examples/clock");
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_cp() {
//     let s = run_and_capture("examples/cp");
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_cp_and_insert() {
//     let s = run_and_capture("examples/cp_and_insert");
//     // assert!(s == "This is the contents of the file!\n");
// }

// TODO: clean up?
#[test]
fn example_fallocate() {
    let s = run_and_capture("examples/fallocate");
    assert!(Path::new("examples/fallocate/output/tmp.txt").exists());
    // assert!(s == "This is the contents of the file!\n");
}

#[test]
fn example_hello() {
    let s = run_and_capture("examples/hello");
    assert!(s == "Hello, World!\n");
}

// #[test]
// fn example_link() {
//     let s = run_and_capture("examples/link");
//     // assert!(s == "This is the contents of the file!\n");
// }

#[test]
fn example_ls() {
    let s = run_and_capture("examples/ls");
    let s_split = s.split("\n").collect::<Vec<&str>>();
    assert!(s_split[0] == "I'm doing an ls(.)!");
    assert!(s_split[1] == "ls    ls.c    ls.wasm.h    ls.wasm.c    ..    .    Makefile    ");
    assert!(s_split[2] == "Done!");
}

// #[test]
// fn example_mkdir() {
//     let s = run_and_capture("examples/mkdir");
//     // assert!(s == "This is the contents of the file!\n");
// }

#[test]
fn example_permissions_regression() {
    let s = run_and_capture("examples/permissions_regression");
    let s_split = s.split("\n").collect::<Vec<&str>>();
    assert!(s_split[0] == "Hello, World!");
    assert!(s_split[1] == "write returns 11");
    // assert!(s == "This is the contents of the file!\n");
}

#[test]
fn example_print_args_and_environ() {
    let s = run_and_capture("examples/print_args_and_environ");
    let s_split = s.split("\n").collect::<Vec<&str>>();
    assert!(s_split[0] == "Number of arguments passed: 3");
    assert!(s_split[1] == "argv[0]: a");
    assert!(s_split[2] == "argv[1]: b");
    assert!(s_split[3] == "argv[2]: c");
    assert!(s_split[4] == "Environment Variable: CC=clang");
    assert!(s_split[5] == "Environment Variable: CXX=clang++");
    assert!(s_split[6] == "Done!");
}

#[test]
fn example_raise() {
    let s = run_and_capture("examples/raise");
    // all we expect is that it doesn't crash
}

// #[test]
// fn example_random() {
//     let s = run_and_capture("examples/random");
//     // let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
//     // assert!(re.is_match(s));
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_rename() {
//     let s = run_and_capture("examples/rename");
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_renumber() {
//     let s = run_and_capture("examples/renumber");
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_rmdir() {
//     let s = run_and_capture("examples/rmdir");
//     // assert!(s == "This is the contents of the file!\n");
// }

#[test]
fn example_setfl() {
    let s = run_and_capture("examples/setfl");
    let s_split = s.split("\n").collect::<Vec<&str>>();
    assert!(s_split[0] == "oldflags = 8000000");
    assert!(s_split[1] == "newflags = 8000400");
    assert!(s_split[2] == "Done!");
    // assert!(s == "This is the contents of the file!\n");
}

#[test]
fn example_sleep() {
    let s = run_and_capture("examples/sleep");
    let s_split = s.split("\n").collect::<Vec<&str>>();
    assert!(s_split[0] == "Please wait for 5 seconds...");
    assert!(s_split[1] == "Thank you for waiting!");
    // assert!(s == "This is the contents of the file!\n");
}

// #[test]
// fn example_stat() {
//     let s = run_and_capture("examples/stat");
//     // assert!(s == "This is the contents of the file!\n");
// }

// #[test]
// fn example_symlink() {
//     let s = run_and_capture("examples/symlink");
//     // assert!(s == "This is the contents of the file!\n");
// }

#[test]
fn example_sync() {
    let s = run_and_capture("examples/sync");
    assert!(s == "Done!\n");
}