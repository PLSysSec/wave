pub mod parser;
pub mod types;
pub mod wrapper_codegen;
use crate::parser::parse_spec_from_file;
use crate::types::{Language, Spec};
use crate::wrapper_codegen::gen_c_wrappers;
use clap::{App, Arg};
use std::fs;

fn lang_from_string(input: String) -> Result<Language, ()> {
    match &*input.to_lowercase() {
        "c" => Ok(Language::C),
        "cpp" => Ok(Language::Cpp),
        "rust" => Ok(Language::Rust),
        _ => Err(()),
    }
}

#[derive(Debug)]
pub struct Config {
    language: Language,
    spec_path: String,
    impl_path: String,
    out_check_path: String,
}

fn run(config: Config) {
    println!("config = {:?}", config);
    let spec = parse_spec_from_file(config.spec_path).unwrap();
    println!("===== Signatures =====");
    for sig in &spec.sigs {
        println!("{:?}", sig);
    }
    println!("===== Policies =====");
    for policy in &spec.policies {
        println!("{:?}", policy);
    }
    let wrappers = gen_c_wrappers(&spec);  
    fs::write(config.impl_path, wrappers);
}

fn main() {
    let matches = App::new("spec_parser")
        .version("0.1.0")
        .about("Parses spec and validates wrapper code against that spec")
        .arg(
            Arg::with_name("language")
                .long("lang")
                .takes_value(true)
                .help("What language to use"),
        )
        .arg(
            Arg::with_name("spec_path")
                .long("spec")
                .takes_value(true)
                .help("path to specification to validate against")
                .required(true),
        )
        .arg(
            Arg::with_name("out_impl_path")
                .long("out_impl")
                .takes_value(true)
                .help("path to outfile for wrappers")
                .required(true),
        )
        .arg(
            Arg::with_name("out_check_path")
                .long("out_check")
                .takes_value(true)
                .help("path to outfile to be checked by smack")
                .required(true),
        )
        .get_matches();

    let language = matches.value_of("language").unwrap();
    let spec_path = matches.value_of("spec_path").unwrap();
    let out_impl_path = matches.value_of("out_impl_path").unwrap();
    let out_check_path = matches.value_of("out_check_path").unwrap();

    let config = Config {
        language: lang_from_string(language.to_string()).unwrap(),
        spec_path: spec_path.to_string(),
        impl_path: out_impl_path.to_string(),
        out_check_path: out_check_path.to_string(),
    };

    run(config);
}
