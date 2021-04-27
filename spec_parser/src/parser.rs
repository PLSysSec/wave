use std::fs;
use std::io;

#[derive(Debug)]
pub struct WrapperSignature {}

#[derive(Debug)]
pub struct WrapperPolicy {}

#[derive(Debug)]
pub struct Spec {
    sigs: Vec<WrapperSignature>,
    policies: Vec<WrapperPolicy>
}


fn parse_wrapper_policy(line: &str) -> WrapperPolicy{
    return WrapperPolicy {};
}

fn parse_wrapper_sig(line: &str) -> WrapperSignature{
    return WrapperSignature {};
}

pub fn parse_spec_from_file(spec_path: String) -> io::Result<Spec>{
    let spec_str = fs::read_to_string("address.txt")?;
    return parse_spec_from_string(spec_str);
}

pub fn parse_spec_from_string(spec_str: String) -> io::Result<Spec>{
    let mut sigs: Vec<WrapperSignature> = Vec::new();
    let mut policies: Vec<WrapperPolicy> = Vec::new();

    for line in spec_str.split("\n"){
        if line.starts_with("Policy"){
            let policy = parse_wrapper_policy(line);
            policies.push(policy);
        }
        else{
            let sig = parse_wrapper_sig(line);
            sigs.push(sig);
        }
    }

    return Ok(Spec {sigs: sigs, policies: policies});
}