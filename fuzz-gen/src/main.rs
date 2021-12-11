#![allow(dead_code)]

#[macro_use]
pub mod parse_quote_spanned;
mod extern_spec_rewriter;
mod parse_closure_macro;
pub mod qc_rewriter;
mod rewriter;
mod span_overrider;
mod spec_attribute_kind;
pub mod specifications;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::TokenStreamExt;
use quote::{quote, quote_spanned, ToTokens};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::read_to_string;
use std::path::Path;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Expr, File, FnArg, Ident, ImplItem,
    ImplItemMethod, Item, ItemFn, Receiver, Result, Signature, Token, Type,
};

use parse_closure_macro::ClosureWithSpec;
pub use spec_attribute_kind::SpecAttributeKind;
use specifications::untyped::{self, AnyFnItem};

macro_rules! force_matches {
    ($ex:expr, $patt:pat => $ret:expr, $err:expr) => {
        if let $patt = $ex {
            $ret
        } else {
            unreachable!($err)
        }
    };
    ($ex:expr, $patt:pat => $ret:expr) => {
        if let $patt = $ex {
            $ret
        } else {
            unreachable!(
                "force_matches: expr {} doesn't match pattern {}",
                stringify!($ex),
                stringify!($patt)
            )
        }
    };
}

#[derive(Debug)]
pub enum CheckableItem {
    Fn(Function),
    Method(Method),
}

#[derive(Debug, Clone)]
pub struct Function {
    func: ItemFn,
}

#[derive(Debug, Clone)]
pub struct Method {
    receiver: Type,
    func: ImplItemMethod,
}

impl CheckableItem {
    pub fn attrs(&self) -> &Vec<syn::Attribute> {
        match self {
            CheckableItem::Fn(item) => &item.func.attrs,
            CheckableItem::Method(item) => &item.func.attrs,
        }
    }

    pub fn attrs_mut(&mut self) -> &mut Vec<syn::Attribute> {
        match self {
            CheckableItem::Fn(item) => &mut item.func.attrs,
            CheckableItem::Method(item) => &mut item.func.attrs,
        }
    }

    pub fn sig(&self) -> &syn::Signature {
        match self {
            CheckableItem::Fn(item) => &item.func.sig,
            CheckableItem::Method(item) => &item.func.sig,
        }
    }

    pub fn block(&self) -> Option<&syn::Block> {
        match self {
            CheckableItem::Fn(item) => Some(&item.func.block),
            CheckableItem::Method(item) => Some(&item.func.block),
        }
    }

    pub fn is_trusted(&self) -> bool {
        //Attribute { pound_token: Pound, style: Outer, bracket_token: Bracket, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident(requires), arguments: None }] }, tokens: TokenStream [Group { delimiter: Parenthesis, stream: TokenStream [Ident { sym: trace_safe }, Group { delimiter: Parenthesis, stream: TokenStream [Ident { sym: trace }, Punct { char: ',', spacing: Alone }, Ident { sym: ctx }, Punct { char: '.', spacing: Alone }, Ident { sym: memlen }] }, Punct { char: '&', spacing: Joint }, Punct { char: '&', spacing: Alone }, Ident { sym: ctx_safe }, Group { delimiter: Parenthesis, stream: TokenStream [Ident { sym: ctx }] }] }] }

        for attr in self.attrs() {
            if attr.path.is_ident("trusted") {
                return true;
            }
        }
        false
    }
}

impl ToTokens for CheckableItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            CheckableItem::Fn(item) => item.func.to_tokens(tokens),
            CheckableItem::Method(item) => item.func.to_tokens(tokens),
        }
    }
}

struct QcModule {
    functions: Vec<QcFunction>,
    impls: HashMap<Type, QcImpl>,
}

impl QcModule {
    fn new() -> QcModule {
        QcModule {
            functions: Vec::new(),
            impls: HashMap::new(),
        }
    }

    fn append(&mut self, item: &mut CheckableItem) {
        let attrs = extract_prusti_attributes(item);
        let (mut mats, pres) = generate_quickcheck_preconditions(attrs.clone(), &item).unwrap();
        let (post_mats, posts) = generate_quickcheck_postconditions(attrs, &item).unwrap();
        mats.extend(post_mats);

        match item {
            CheckableItem::Fn(fn_item) => {
                let qc_func = QcFunction {
                    mats,
                    pres,
                    posts,
                    func: fn_item.clone(),
                };
                self.functions.push(qc_func);
            }
            CheckableItem::Method(method_item) => {
                let qc_method = QcMethod {
                    mats,
                    pres,
                    posts,
                    func: method_item.clone(),
                };
                let receiver = &method_item.receiver;
                if let Some(imp) = self.impls.get_mut(receiver) {
                    imp.methods.push(qc_method);
                } else {
                    self.impls.insert(
                        receiver.clone(),
                        QcImpl {
                            ty: receiver.clone(),
                            methods: vec![qc_method],
                        },
                    );
                }
            }
            _ => {}
        }
    }
}

impl ToTokens for QcModule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for function in &self.functions {
            function.to_tokens(tokens);
        }
        for (_, imp) in &self.impls {
            imp.to_tokens(tokens);
        }
    }
}

struct QcFunction {
    mats: TokenStream,
    pres: Vec<Expr>,
    posts: Vec<Expr>,
    func: Function,
}

impl ToTokens for QcFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fn_name = &self.func.func.sig.ident;
        let qc_name = syn::Ident::new(&format!("check_{}", fn_name), fn_name.span());
        let args = &self.func.func.sig.inputs;
        let (convs, new_args) = to_qc_args(args);
        let mats = &self.mats;
        let pres = &self.pres;
        let posts = &self.posts;
        let func_call = &self.func.func.sig.ident;
        let call_args = args_to_arglist(&args);
        tokens.extend(quote! {
            #[quickcheck_macros::quickcheck]
            fn #qc_name(#new_args) -> TestResult {
                init(); // must not panic...
                #convs
                #(#pres)*
                #mats
                let result = #func_call(#(#call_args),*);
                #(#posts)*
                TestResult::passed()
            }
        });
    }
}

struct QcImpl {
    ty: Type,
    methods: Vec<QcMethod>,
}

impl ToTokens for QcImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let methods = &self.methods;
        for method in methods {
            method.add_qc_thunk(tokens);
        }
        let ty = &self.ty;
        tokens.extend(quote! {
            impl #ty {
                #(#methods)*
            }
        });
    }
}

struct QcMethod {
    mats: TokenStream,
    pres: Vec<Expr>,
    posts: Vec<Expr>,
    func: Method,
}

impl QcMethod {
    fn thunk_ident(&self) -> Ident {
        let fn_name = &self.func.func.sig.ident;
        syn::Ident::new(&format!("check_{}", fn_name), fn_name.span())
    }

    fn thunk_args(&self) -> Punctuated<FnArg, Comma> {
        self.func.func.sig.inputs.iter().skip(1).cloned().collect()
    }

    fn method_ident(&self) -> Ident {
        let fn_name = &self.func.func.sig.ident;
        syn::Ident::new(&format!("check_{}_impl", fn_name), fn_name.span())
    }

    fn add_qc_thunk(&self, tokens: &mut TokenStream) {
        let name = self.thunk_ident();
        let args = self.thunk_args();
        let (convs, new_args) = to_qc_args(&args);
        let method_ident = self.method_ident();
        let call_args = args_to_arglist(&args);

        tokens.extend(quote! {
            #[quickcheck_macros::quickcheck]
            fn #name(#new_args) -> TestResult {
                init(); // must not panic...
                #convs
                let mut ctx = fresh_ctx(".".to_string());
                ctx.#method_ident(#(#call_args),*)
            }
        });
    }
}

impl ToTokens for QcMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.method_ident();
        let args = &self.func.func.sig.inputs;
        let mats = &self.mats;
        let pres = &self.pres;
        let posts = &self.posts;
        let func_call = &self.func.func.sig.ident;
        let call_args = args_to_arglist(&args);
        tokens.extend(quote! {
            fn #name(#args) -> TestResult {
                #(#pres)*
                #mats
                let result = self.#func_call(#(#call_args),*);
                #(#posts)*
                TestResult::passed()
            }
        });
    }
}

/// Converts an argument list into a list that can be supplied by QuickCheck, and also returns
/// a TokenStream of converters that will convert the new args to the proper form.
fn to_qc_args(args: &Punctuated<FnArg, Comma>) -> (TokenStream, Punctuated<FnArg, Comma>) {
    let mut res = Punctuated::new();
    let mut convs = TokenStream::new();
    for arg in args {
        match arg {
            FnArg::Receiver(..) => {
                unimplemented!("to_qc_args should not be given `self`! Don't pass in method args!")
            }
            FnArg::Typed(pat_type) => {
                println!("{:?}", pat_type.pat);
                match &*pat_type.ty {
                    Type::Reference(reference) => {
                        let inner = &*reference.elem;
                        let muta = reference.mutability;
                        let res_type = match inner {
                            Type::Slice(slic) => {
                                let elem = &*slic.elem;
                                Type::Verbatim(quote::quote! {Vec<#elem>})
                            }
                            _ => inner.clone(),
                        };
                        let pat_ident = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                            ident
                        } else {
                            unimplemented!("to_qc_args expects simple identifiers for args!")
                        };
                        let new_ident = Ident::new(
                            format!("vec_{}", pat_ident.ident.to_string()).as_ref(),
                            pat_ident.span(),
                        );
                        let res_pat = syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: muta,
                            ident: new_ident.clone(),
                            subpat: None,
                        });
                        let res_arg = FnArg::Typed(syn::PatType {
                            attrs: vec![],
                            pat: Box::new(res_pat),
                            colon_token: pat_type.colon_token,
                            ty: Box::new(res_type),
                        });
                        res.push(res_arg);
                        let ident = &pat_ident.ident;
                        let converter = match inner {
                            Type::Slice(slic) => {
                                quote! {
                                    let #ident = &#muta #new_ident[..];
                                }
                            }
                            _ => quote! {let #ident = &#muta #new_ident;},
                        };
                        convs.extend(converter);
                    }
                    // if not reference, just keep the same
                    _ => res.push(FnArg::Typed(pat_type.clone())),
                }
            }
        }
    }
    (convs, res)
}

fn args_to_arglist(args: &Punctuated<FnArg, Comma>) -> Vec<syn::Pat> {
    let mut res = vec![];
    for arg in args {
        match arg {
            FnArg::Receiver(..) => {
                // ignore
            }
            FnArg::Typed(pat_type) => res.push(*pat_type.pat.clone()),
        }
    }
    res
}

fn main() -> std::io::Result<()> {
    // TODO: problems:
    // 1. need to handle fresh context...
    // 2.
    // 1. no after_expiry support
    //let contents = read_to_string("test.rs")?;
    //let tokens: proc_macro2::TokenStream = contents.parse().unwrap();
    //let file: syn::File = syn::parse2(tokens).unwrap();

    //// convert functions and methods into common type for processing...
    //let mut fns = vec![];
    //for item in file.items.into_iter() {
    //    match item {
    //        syn::Item::Fn(item_fn) => fns.push(CheckableItem::Fn(Function { func: item_fn })),
    //        syn::Item::Impl(item_impl) => {
    //            for impl_item in item_impl.items {
    //                match impl_item {
    //                    ImplItem::Method(method) => fns.push(CheckableItem::Method(Method {
    //                        receiver: *item_impl.self_ty.clone(),
    //                        func: method,
    //                    })),
    //                    _ => {}
    //                }
    //            }
    //        }
    //        _ => {}
    //    }
    //}

    let mut qc_module = QcModule::new();
    let src_path = Path::new("../src");
    process_directory(&src_path, &mut qc_module)?;
    ////let quickcheck_functions = vec![];
    //for mut ffn in fns {
    //    qc_module.append(&mut ffn);
    //}

    let tokens = quote! {
        #qc_module
    };
    println!("{:?}", tokens.to_string());

    Ok(())
}

fn process_directory(path: &Path, qc: &mut QcModule) -> std::io::Result<()> {
    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            process_directory(&path, qc);
        } else {
            process_file(&path, qc);
        }
    }

    Ok(())
}

fn process_file(file_path: &Path, qc: &mut QcModule) -> std::io::Result<()> {
    // skip trace file...
    if file_path.to_str().unwrap().contains("trace") {
        return Ok(());
    }

    let contents = read_to_string(file_path)?;
    let tokens: proc_macro2::TokenStream = contents.parse().unwrap();
    let file: syn::File = syn::parse2(tokens).unwrap();

    // convert functions and methods into common type for processing...
    let mut fns = vec![];
    for item in file.items.into_iter() {
        match item {
            syn::Item::Fn(item_fn) => fns.push(CheckableItem::Fn(Function { func: item_fn })),
            syn::Item::Impl(item_impl) => {
                for impl_item in item_impl.items {
                    match impl_item {
                        ImplItem::Method(method) => fns.push(CheckableItem::Method(Method {
                            receiver: *item_impl.self_ty.clone(),
                            func: method,
                        })),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    //let quickcheck_functions = vec![];
    for mut ffn in fns {
        if ffn.is_trusted() {
            qc.append(&mut ffn);
        }
    }

    Ok(())

}

pub type GeneratedResult = syn::Result<Vec<syn::Expr>>;

fn generate_quickcheck_preconditions(
    mut prusti_attributes: Vec<(SpecAttributeKind, TokenStream)>,
    item: &CheckableItem,
) -> syn::Result<(TokenStream, Vec<syn::Expr>)> {
    let mut generated_items = vec![];
    let mut generated_mats = TokenStream::new();
    //let mut generated_attributes = vec![];

    for (attr_kind, attr_tokens) in prusti_attributes.drain(..) {
        let rewriting_result: syn::Result<(TokenStream, Vec<syn::Expr>)> = match attr_kind {
            SpecAttributeKind::Requires => generate_qc_for_requires(attr_tokens, item),
            //SpecAttributeKind::Ensures => generate_qc_for_ensures(attr_tokens, item),
            /*SpecAttributeKind::AfterExpiry => generate_for_after_expiry(attr_tokens, item),
            SpecAttributeKind::AfterExpiryIf => generate_for_after_expiry_if(attr_tokens, item),
            SpecAttributeKind::Pure => generate_for_pure(attr_tokens, item),
            SpecAttributeKind::Trusted => generate_for_trusted(attr_tokens, item),
            // Predicates are handled separately below; the entry in the SpecAttributeKind enum
            // only exists so we successfully parse it and emit an error in
            // `check_incompatible_attrs`; so we'll never reach here.
            SpecAttributeKind::Predicate => unreachable!(),*/
            _ => Ok((TokenStream::new(), vec![])), // do nothing for non requires / ensures
        };
        let (mats, new_items) = rewriting_result?;
        generated_items.extend(new_items);
        generated_mats.extend(mats);
    }

    Ok((generated_mats, generated_items))
}

fn generate_quickcheck_postconditions(
    mut prusti_attributes: Vec<(SpecAttributeKind, TokenStream)>,
    item: &CheckableItem,
) -> syn::Result<(TokenStream, Vec<syn::Expr>)> {
    let mut generated_items = vec![];
    let mut generated_mats = TokenStream::new();
    //let mut generated_attributes = vec![];

    for (attr_kind, attr_tokens) in prusti_attributes.drain(..) {
        let rewriting_result: syn::Result<(TokenStream, Vec<syn::Expr>)> = match attr_kind {
            //SpecAttributeKind::Requires => generate_qc_for_requires(attr_tokens, item),
            SpecAttributeKind::Ensures => generate_qc_for_ensures(attr_tokens, item),
            /*SpecAttributeKind::AfterExpiry => generate_for_after_expiry(attr_tokens, item),
            SpecAttributeKind::AfterExpiryIf => generate_for_after_expiry_if(attr_tokens, item),
            SpecAttributeKind::Pure => generate_for_pure(attr_tokens, item),
            SpecAttributeKind::Trusted => generate_for_trusted(attr_tokens, item),
            // Predicates are handled separately below; the entry in the SpecAttributeKind enum
            // only exists so we successfully parse it and emit an error in
            // `check_incompatible_attrs`; so we'll never reach here.
            SpecAttributeKind::Predicate => unreachable!(),*/
            _ => Ok((TokenStream::new(), vec![])), // do nothing for non requires / ensures
        };
        let (mats, new_items) = rewriting_result?;
        generated_items.extend(new_items);
        generated_mats.extend(mats);
        //generated_attributes.extend(new_attributes);
    }

    Ok((generated_mats, generated_items))
}

/// Generate spec items and attributes to typecheck the and later retrieve "requires" annotations.
fn generate_qc_for_requires(
    attr: TokenStream,
    item: &CheckableItem,
) -> syn::Result<(TokenStream, Vec<syn::Expr>)> {
    let mut rewriter = qc_rewriter::QcAstRewriter::new();
    let assertion = rewriter.parse_assertion(attr)?;
    let (mats, spec_item) =
        rewriter.generate_qc_expr(qc_rewriter::SpecItemType::Precondition, assertion, item)?;
    Ok((mats, vec![spec_item]))
}

/// Generate spec items and attributes to typecheck the and later retrieve "ensures" annotations.
fn generate_qc_for_ensures(
    attr: TokenStream,
    item: &CheckableItem,
) -> syn::Result<(TokenStream, Vec<syn::Expr>)> {
    let mut rewriter = qc_rewriter::QcAstRewriter::new();
    let assertion = rewriter.parse_assertion(attr)?;
    let (mats, spec_item) =
        rewriter.generate_qc_expr(qc_rewriter::SpecItemType::Postcondition, assertion, item)?;
    Ok((mats, vec![spec_item]))
}

pub fn extract_prusti_attributes(
    item: &mut CheckableItem,
) -> Vec<(SpecAttributeKind, TokenStream)> {
    let mut prusti_attributes = Vec::new();
    let mut regular_attributes = Vec::new();
    for attr in item.attrs_mut().drain(0..) {
        if attr.path.segments.len() == 1 {
            if let Ok(attr_kind) = attr.path.segments[0].ident.to_string().try_into() {
                let tokens = match attr_kind {
                    SpecAttributeKind::Requires
                    | SpecAttributeKind::Ensures
                    | SpecAttributeKind::AfterExpiry
                    | SpecAttributeKind::AfterExpiryIf => {
                        // We need to drop the surrounding parenthesis to make the
                        // tokens identical to the ones passed by the native procedural
                        // macro call.
                        let mut iter = attr.tokens.into_iter();
                        let tokens = force_matches!(iter.next().unwrap(), TokenTree::Group(group) => group.stream());
                        assert!(iter.next().is_none(), "Unexpected shape of an attribute.");
                        tokens
                    }
                    // Nothing to do for attributes without arguments.
                    SpecAttributeKind::Pure
                    | SpecAttributeKind::Trusted
                    | SpecAttributeKind::Predicate => {
                        assert!(attr.tokens.is_empty(), "Unexpected shape of an attribute.");
                        attr.tokens
                    }
                };
                prusti_attributes.push((attr_kind, tokens));
            } else {
                regular_attributes.push(attr);
            }
        } else {
            regular_attributes.push(attr);
        }
    }
    *item.attrs_mut() = regular_attributes;
    prusti_attributes
}
