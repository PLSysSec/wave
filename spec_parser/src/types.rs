use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub enum Language {
    C,
    Cpp,
    Rust,
}

#[derive(Debug)]
pub struct WrapperSignature {
    pub function_name: String,
    pub ret_ty: Ctype,
    pub args: Vec<(Ctype, String)>,
}

#[derive(Debug)]
pub struct WrapperPolicy {
    pub function_name: String,
    pub annotations: Vec<(String, TypeQualifier)>,
}

#[derive(Debug)]
pub struct Spec {
    pub sigs: Vec<WrapperSignature>,
    pub policies: Vec<WrapperPolicy>,
}

#[derive(Debug)]
pub enum Ctype {
    Char,
    Int,
    Void,
    SizeT,
    SsizeT,
    Pointer(Box<Ctype>, bool), //bool = mutable
}

impl FromStr for Ctype {
    type Err = ();
    fn from_str(input: &str) -> Result<Ctype, Self::Err> {
        match input {
            "char" => Ok(Ctype::Char),
            "int" => Ok(Ctype::Int),
            "void" => Ok(Ctype::Void),
            "size_t" => Ok(Ctype::SizeT),
            "ssize_t" => Ok(Ctype::SsizeT),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum TypeQualifier {
    Qualifier0Arg(String),
    Qualifier1Arg(String, String),
}
