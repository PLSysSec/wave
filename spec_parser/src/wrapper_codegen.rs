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

fn gen_syscall(sig: &WrapperSignature) -> String{
    let syscall_str = format!("return syscall(SYS_{:?}, 
        {} 
        NULL);
    ", sig.function_name, "fd,");
    return syscall_str;
}

fn gen_c_wrapper(sig: &WrapperSignature, policy: &WrapperPolicy) -> String{
    let sig_str = sig.to_string();
    let syscall_str = gen_syscall(sig);
    format!("{} {{
                {}
                }}\n", 
    sig_str, syscall_str)
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
