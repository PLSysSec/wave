use std::str::FromStr;
use std::collections::HashMap;

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

impl ToString for WrapperSignature {
    fn to_string(&self) -> String {
        let args_str = self.args.iter()
            .map(|(ty,name)| ty.to_string() + " " + name)
            .collect::<Vec<String>>()
            .join(", ");
        return format!("{:} {:} ({:})", self.ret_ty.to_string(), self.function_name, args_str);  
    }
}

#[derive(Debug)]
pub struct WrapperPolicy {
    pub function_name: String,
    pub annotations: Vec<(String, TypeQualifier)>,
}

#[derive(Debug)]
pub struct Spec {
    pub sigs: HashMap<String, WrapperSignature>,
    pub policies: HashMap<String, WrapperPolicy>,
}

#[derive(Debug)]
pub enum Ctype {
    Char,
    Int,
    Void,
    SizeT,
    SsizeT,
    OffT,
    ModeT,
    Pointer(Box<Ctype>, bool), //bool = mutable
    CStruct(String), // string name
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
            "off_t" => Ok(Ctype::OffT),
            "mode_t" => Ok(Ctype::ModeT),
            _ => Err(()),
        }
    }
}

impl ToString for Ctype {
    fn to_string(&self) -> String {
        match self {
            Ctype::Char => "char".to_string(),
            Ctype::Int => "int".to_string(),
            Ctype::Void => "void".to_string(),
            Ctype::SizeT => "size_t".to_string(),
            Ctype::SsizeT => "ssize_t".to_string(),
            Ctype::OffT => "off_t".to_string(),
            Ctype::ModeT => "mode_t".to_string(),
            Ctype::Pointer(ptr, is_mut) => {
                if *is_mut {
                    format!("* {:}", (*ptr).to_string())
                }
                else {
                    format!("const * {:}", (*ptr).to_string())
                }
            },
            Ctype::CStruct(name) => name.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum TypeQualifier {
    Qualifier0Arg(String),
    Qualifier1Arg(String, String),
}
