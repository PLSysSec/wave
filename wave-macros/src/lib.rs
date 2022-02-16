#![feature(proc_macro_diagnostic)]
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream, Result};
//use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Expr, ExprCall, ExprMethodCall, ExprPath, FnArg, Ident, ItemFn,
    Meta, NestedMeta, Pat, Path, PathArguments, PathSegment, Signature, 
};

/// Right now macro should be used like:
/// #[with_extra_arg(trace: &mut Trace)]

struct Args {
    decl: FnArg,
    var: Ident,
    //ty: Type,
    external_calls: Vec<Ident>,
    external_methods: Vec<Ident>,
}

impl Args {
    fn decl(&self) -> FnArg {
        self.decl.clone()
    }

    fn var(&self) -> Ident {
        self.var.clone()
    }

    /// Converts the state ident to an expression that can be used as an argument to functions
    fn var_as_expr(&self) -> Expr {
        let var = self.var();
        let segment = PathSegment {
            ident: var,
            arguments: PathArguments::None,
        };
        let path = Path::from(segment);
        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };
        let expr = Expr::from(expr_path);
        expr
    }

    //fn ty(&self) -> Type {
    //    self.ty.clone()
    //}
}

fn pat_to_ident(pat: Pat) -> Result<Ident> {
    if let Pat::Ident(pat_ident) = pat {
        return Ok(pat_ident.ident);
    }

    Err(syn::Error::new(
        pat.span(),
        "Argument needs to be bound to an identifier",
    ))
}

fn path_to_last_ident(path: Path) -> Result<Ident> {
    let err = Err(syn::Error::new(
        path.span(),
        "Expr needs to be an ident to call expr_to_ident",
    ));

    let final_segment = path.segments.last();
    return final_segment.map_or(err, |i| Ok(i.ident.clone()));
}

/// MyClass::new() => Ok(new)
/// do_math(x) => Ok(do_math)
fn expr_to_ident(expr: Expr) -> Result<Ident> {
    // if it is an ident, try to unwrap and return it
    if let Expr::Path(expr_path) = expr {
        return path_to_last_ident(expr_path.path);
    }
    // else, fail
    Err(syn::Error::new(
        expr.span(),
        "Expr needs to be an ident to call expr_to_ident",
    ))
}

// Parse arguments for macro
impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let decl = syn::FnArg::parse(input)?;
        if let FnArg::Typed(ref pat_ty) = decl {
            let var = pat_to_ident(*pat_ty.pat.clone())?;
            //let ty = *pat_ty.ty.clone();
            return Ok(Args {
                decl,
                var,
                //ty,
                // Hardcoded external calls
                external_calls: vec![
                    // Common calls
                    Ident::new("Ok", Span::call_site()),
                    Ident::new("Err", Span::call_site()),
                    Ident::new("new", Span::call_site()),
                    // Instrumentation
                    Ident::new("start_timer", Span::call_site()),
                    Ident::new("stop_timer", Span::call_site()),
                    Ident::new("push_syscall_result", Span::call_site()),
                    Ident::new("push_hostcall_result", Span::call_site()),
                ],
                // Hardcoded External methods
                external_methods: vec![
                    // Common methods
                    Ident::new("into", Span::call_site()),
                    Ident::new("len", Span::call_site()),
                    Ident::new("ok_or", Span::call_site()),
                ],
            });
        }

        Err(syn::Error::new(
            decl.span(),
            "You can't use self as an extra arg.",
        ))
    }
}

impl Fold for Args {
    /// Rewrite function signature of the form ... func(arg:Type,...) =>  func(arg:Type,..., state: &mut GlobalState)
    fn fold_signature(&mut self, sig: Signature) -> Signature {
        let arg_decl = self.decl(); // FnArg
        let mut new_sig = sig;
        new_sig.inputs.push(arg_decl);
        new_sig
    }

    /// Rewrite calls of the form func(...) => func(..., state)
    /// If the call is in self.external_calls, ignore it
    fn fold_expr_call(&mut self, call: ExprCall) -> ExprCall {
        //println!("==> Rewriting fn call {:#?}", call);
        //println!("External calls: {:#?}", self.external_calls);
        // if call is external, ignore it
        for external_call in self.external_calls.iter() {
            let func_name: Result<Ident> = expr_to_ident(*call.func.clone());
            match func_name {
                Ok(name) => {
                    if name.to_string() == external_call.to_string() {
                        //println!("<== Call to {:#?} is external, skipping", call);
                        return call;
                    }
                }
                Err(_) => {}
            }
        }
        // else, add ghost state
        let arg = self.var_as_expr();
        let mut new_call = call;
        new_call.args.push(arg);
        //println!("<== Call instrumented: {:#?}", new_call);
        new_call
    }

    /// Rewrite method calls of the form o.method(...) => o.method(..., state)
    /// If the call is in self.external_methods, ignore it
    fn fold_expr_method_call(&mut self, method_call: ExprMethodCall) -> ExprMethodCall {
        //println!("==> Rewriting method call {:#?}", method_call);
        //println!("External methods: {:#?}", self.external_methods);
        let mut new_method_call = method_call;
        *new_method_call.receiver = self.fold_expr(*new_method_call.receiver);

        // If method is external, ignore it
        for external_method in self.external_methods.iter() {
            if new_method_call.method.to_string() == external_method.to_string() {
                //println!("<== Call to {:#?} is external, skipping", new_method_call);
                return new_method_call;
            }
        }
        // else, add ghost state
        let arg = self.var_as_expr();
        new_method_call.args.push(arg);
        //println!("<== Method call instrumented: {:#?}", new_method_call);
        new_method_call
    }

    /// Record external_call and external_method attributes
    /// self.external_calls (and self.external_methods) currently just store
    /// the name of the target function/method
    fn fold_attribute(&mut self, attr: Attribute) -> Attribute {
        if attr.path.is_ident("external_call") {
            let func_name: Result<Ident> = attr.parse_args();
            match func_name {
                Ok(name) => self.external_calls.push(name),
                Err(_) => return attr,
            };
        }

        if attr.path.is_ident("external_method") {
            let method_name: Result<Ident> = attr.parse_args();
            match method_name {
                Ok(name) => self.external_methods.push(name),
                Err(_) => return attr,
            };
        }

        if attr.path.is_ident("external_calls") {
            let func_names: Result<Meta> = attr.parse_meta();
            if let Ok(Meta::List(l)) = func_names {
                // iterate over punctuated list of elements
                for meta in l.nested.iter() {
                    if let NestedMeta::Meta(m2) = meta {
                        let name = path_to_last_ident(m2.path().clone()).unwrap();
                        self.external_calls.push(name.clone());
                    }
                }
            }
        }

        if attr.path.is_ident("external_methods") {
            let func_names: Result<Meta> = attr.parse_meta();
            if let Ok(Meta::List(l)) = func_names {
                // iterate over punctuated list of elements
                for meta in l.nested.iter() {
                    if let NestedMeta::Meta(m2) = meta {
                        let name = path_to_last_ident(m2.path().clone()).unwrap();
                        self.external_methods.push(name.clone());
                    }
                }
            }
        }
        // match func_names {
        //     Ok(names) => {
        //         for name in names.iter(){
        //             self.external_calls.push(name.clone())
        //         }
        //     },
        //     Err(_) => return attr,
        // };
        //}

        attr
    }
}

#[cfg(feature = "enable")]
#[proc_macro_attribute]
pub fn with_ghost_var(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    // Parse the name binding and type of the global state
    let mut args = parse_macro_input!(args as Args);

    // Use a syntax tree traversal to transform the function body and signature.
    let output = args.fold_item_fn(input);

    // Hand the resulting function body back to the compiler.
    TokenStream::from(quote!(#output))
}

// if extra_arg is disabled, make a noop
#[cfg(not(feature = "enable"))]
#[proc_macro_attribute]
pub fn with_ghost_var(args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn external_call(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn external_method(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn external_calls(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn external_methods(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

// #[proc_macro_attribute]
// pub fn dummy_macro(args: TokenStream, input: TokenStream) -> TokenStream {
//     println!("args: \"{}\"", args.to_string());
//     println!("input: \"{}\"", input.to_string());
//     input
// }

// #[proc_macro_attribute]
// pub fn fuzz_trusted(args: TokenStream, input: TokenStream) -> TokenStream {
//     println!("args: \"{}\"", args.to_string());
//     println!("input: \"{}\"", input.to_string());

//     // // Parse the input tokens into a syntax tree
//     // let input = parse_macro_input!(input as DeriveInput);

//     // Build the output, possibly using quasi-quotation
//     let expanded = quote! {
//         input.to_string()
//         input.to_string()
//     };

//     // Hand the output tokens back to the compiler
//     TokenStream::from(expanded)
//     input
// }

// #[cfg(feature = "enable")]
// #[proc_macro_attribute]
// pub fn all_with_extra_arg(args: TokenStream, input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as ItemFn);
//     // Parse the name binding and type of the global state
//     let mut args = parse_macro_input!(args as Args);

//     // Use a syntax tree traversal to transform the function body and signature.
//     println!("all_with_extra_args: input = {:#?}", input);
//     unimplemented!()
//     // let output = args.fold_item_fn(input);

//     // // Hand the resulting function body back to the compiler.
//     // TokenStream::from(quote!(#output))
// }

// // if extra_arg is disabled, make a noop
// #[cfg(not(feature = "enable"))]
// #[proc_macro_attribute]
// pub fn all_with_extra_arg(args: TokenStream, input: TokenStream) -> TokenStream {
//     input
// }
