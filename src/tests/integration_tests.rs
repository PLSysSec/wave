use std::process::Command;

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