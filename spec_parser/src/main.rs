pub mod parser;
use nom::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  sequence::tuple
};
use clap::{App, Arg};
use parser::parse_spec_from_file;

#[derive(PartialEq, Debug)]
pub enum Language {
    C,
    Cpp,
    Rust,
}

fn lang_from_string(input: String) -> Result<Language, ()> {
    match &*input.to_lowercase() {
        "c"     =>  Ok(Language::C),
        "cpp"   =>  Ok(Language::Cpp),
        "rust"  =>  Ok(Language::Rust),
        _       =>  Err(()),
    }
}

#[derive(Debug)]
pub struct Config {
    language: Language,
    spec_path: String,
    impl_path: String,
}

fn run(config: Config) {
    println!("config = {:?}", config);
    let spec = parse_spec_from_file(config.spec_path);

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
            Arg::with_name("impl_path")
                .long("impl")
                .takes_value(true)
                .help("path to wrapper implementation to validate")
                .required(true),
        )
        .get_matches();

    let language = matches.value_of("language").unwrap();
    let spec_path = matches.value_of("spec_path").unwrap();
    let impl_path = matches.value_of("impl_path").unwrap();

    let config = Config {
        language: lang_from_string(language.to_string()).unwrap(),
        spec_path: spec_path.to_string(),
        impl_path: impl_path.to_string(),
    };

    run(config);
}