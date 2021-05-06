use crate::types::*;

// fn gen_syscall_arg_str(sig: WrapperSignature){
//     match sig.args.len(){
//         0 => "",
//         1 => {

//         }
//         2 => {

//         }
//         3 => {

//         }
//         4 =>
//         _ => panic!("syscall has too many args?")
//     }
// }

fn gen_syscall(sig: &WrapperSignature, args: Vec<String>) -> String{
    let syscall_str = format!("
    return syscall(SYS_{:}, 
        {} 
        NULL);
    ", sig.function_name, args.join(",\n\t"));
    return syscall_str;
}

fn gen_path_sanitizer(input: String) -> (String,String){
    (format!("
    hostptr host_{} = path_from_sandbox({});
    if (host_{} == NULL)
        return -1;", input, input, input),
    format!("host_{}", input))
}

fn gen_sized_buf_sanitizer(input: String, arg: String) -> (String,String){
    (format!("
    hostptr host_{} = sized_buf_from_sandbox({}, {});
    if (host_{} == NULL)
        return -1;", input, input, arg, input), 
    format!("host_{}", input))
}

fn gen_sanitizer(input: String, rule: &TypeQualifier) -> (String,String){
    match rule{
        TypeQualifier::Qualifier0Arg(name) => {
            if name == "PathType"{
                return gen_path_sanitizer(input);
            }
            panic!("Unknown annotation type")
        },
        TypeQualifier::Qualifier1Arg(name, arg) => {
            if name == "SizedBuf"{
                return gen_sized_buf_sanitizer(input, arg.to_string());
            }
            panic!("Unknown annotation type")
        }
    }
}

fn gen_sanitizers(policy: &WrapperPolicy) -> (String,Vec<String>){
    let mut sanitizers_str = "".to_string();
    let mut sanitized_args: Vec<String> = Vec::new();
    for (input,rule) in &policy.annotations{
        let (sanitizer_str, sanitized_arg) = gen_sanitizer(input.to_string(), rule);
        sanitizers_str.push_str(&sanitizer_str); 
        sanitized_args.push(sanitized_arg);
    }
    return (sanitizers_str, sanitized_args);
}

fn gen_c_wrapper(sig: &WrapperSignature, policy: &WrapperPolicy) -> String{
    let sig_str = sig.to_string();
    // let sanitized_args: Vec<String> = Vec::new();
    let (sanitizers_str,sanitized_args) = gen_sanitizers(&policy);

    let syscall_str = gen_syscall(sig, sanitized_args);
    format!("{} {{
                {}
                {}
    }}\n", 
    sig_str, sanitizers_str, syscall_str)
}

pub fn gen_c_wrappers(spec: &Spec) -> String {
    let mut wrappers_str = "".to_string();
    for (fname,sig) in &spec.sigs{
        let policy = spec.policies.get(fname).unwrap();
        let wrapper = gen_c_wrapper(sig, policy);
        wrappers_str.push_str(&wrapper); 
    }
    return wrappers_str;
}
