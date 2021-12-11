use crate::specifications::common::{self, ExpressionIdGenerator, SpecificationIdGenerator};
use crate::specifications::preparser::{Arg, AssertionWithoutId, Parser};
use crate::specifications::untyped::{self, EncodeTypeCheck};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote_spanned};
use std::fmt::Debug;
use syn::spanned::Spanned;
use syn::{punctuated::Punctuated, Expr, ExprPath, Pat, Token, Type};

pub(crate) struct QcAstRewriter {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecItemType {
    Precondition,
    Postcondition,
    Predicate,
}

impl std::fmt::Display for SpecItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecItemType::Precondition => write!(f, "pre"),
            SpecItemType::Postcondition => write!(f, "post"),
            SpecItemType::Predicate => write!(f, "pred"),
        }
    }
}

impl QcAstRewriter {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse an assertion.
    pub fn parse_assertion(&mut self, tokens: TokenStream) -> syn::Result<AssertionWithoutId> {
        let mut parser = Parser::from_token_stream(tokens);
        parser.extract_assertion()
    }

    /*/// Parse a pledge.
    pub fn parse_pledge(
        &mut self,
        spec_id_lhs: Option<untyped::SpecificationId>,
        spec_id_rhs: untyped::SpecificationId,
        tokens: TokenStream
    ) -> syn::Result<untyped::Pledge> {
        untyped::Pledge::parse(tokens, spec_id_lhs, spec_id_rhs, &mut self.expr_id_generator)
    }*/

    fn generate_result_arg(&self, item: &super::CheckableItem) -> syn::FnArg {
        let item_span = item.span();
        let output_ty = match &item.sig().output {
            syn::ReturnType::Default => parse_quote_spanned!(item_span=> ()),
            syn::ReturnType::Type(_, ty) => ty.clone(),
        };
        let fn_arg = syn::FnArg::Typed(syn::PatType {
            attrs: Vec::new(),
            pat: Box::new(parse_quote_spanned!(item_span=> result)),
            colon_token: syn::Token![:](item.sig().output.span()),
            ty: output_ty,
        });
        fn_arg
    }

    /// Generate a quickcheck guard for checking the given precondition, postcondition or predicate.
    ///
    /// `spec_type` should be either `"pre"`, `"post"` or `"pred"`.
    /// // TODO: make this quickcheck func...
    pub fn generate_qc_expr(
        &mut self,
        spec_type: SpecItemType,
        assertion: AssertionWithoutId,
        item: &super::CheckableItem,
    ) -> syn::Result<(TokenStream, syn::Expr)> {
        let item_span = item.span();
        let (materializations, mut statements) = assertion.encode_quick_check();
        let quickcheck = match spec_type {
            SpecItemType::Precondition => quote::quote! {TestResult::discard()},
            SpecItemType::Postcondition => quote::quote! {TestResult::failed()},
            _ => unimplemented!(),
        };

        let mut spec_item: syn::Expr = syn::parse_quote! {
                if !(#statements) {
                    return #quickcheck;
                }
        };
        Ok((materializations, spec_item))
    }

    /*/// Generate statements for checking the given loop invariant.
    pub fn generate_spec_loop(
        &mut self,
        spec_id: untyped::SpecificationId,
        assertion: untyped::Assertion,
    ) -> TokenStream {
        let mut statements = TokenStream::new();
        assertion.encode_type_check(&mut statements);
        let spec_id_str = spec_id.to_string();
        let assertion_json = crate::specifications::json::to_json_string(&assertion);
        let callsite_span = Span::call_site();
        quote_spanned! {callsite_span=>
            #[allow(unused_must_use, unused_variables)]
            {
                #[prusti::spec_only]
                #[prusti::loop_body_invariant_spec]
                #[prusti::spec_id = #spec_id_str]
                #[prusti::assertion = #assertion_json]
                || {
                    #statements
                };
            }
        }
    }

    /// Generate statements for checking a closure specification.
    /// TODO: arguments, result (types are typically not known yet after parsing...)
    pub fn generate_cl_spec(
        &mut self,
        inputs: Punctuated<Pat, Token![,]>,
        output: Type,
        preconds: Vec<(untyped::SpecificationId, untyped::Assertion)>,
        postconds: Vec<(untyped::SpecificationId, untyped::Assertion)>
    ) -> (TokenStream, TokenStream) {
        let process_cond = |is_post: bool, id: &untyped::SpecificationId,
                            assertion: &untyped::Assertion| -> TokenStream
        {
            let spec_id_str = id.to_string();
            let mut encoded = TokenStream::new();
            assertion.encode_type_check(&mut encoded);
            let assertion_json = crate::specifications::json::to_json_string(assertion);
            let name = format_ident!("prusti_{}_closure_{}", if is_post { "post" } else { "pre" }, spec_id_str);
            let callsite_span = Span::call_site();
            let result = if is_post && !inputs.empty_or_trailing() {
                quote_spanned! { callsite_span => , result: #output }
            } else if is_post {
                quote_spanned! { callsite_span => result: #output }
            } else {
                TokenStream::new()
            };
            quote_spanned! { callsite_span =>
                #[prusti::spec_only]
                #[prusti::spec_id = #spec_id_str]
                #[prusti::assertion = #assertion_json]
                fn #name(#inputs #result) {
                    #encoded
                }
            }
        };

        let mut pre_ts = TokenStream::new();
        for (id, precond) in preconds {
            pre_ts.extend(process_cond(false, &id, &precond));
        }

        let mut post_ts = TokenStream::new();
        for (id, postcond) in postconds {
            post_ts.extend(process_cond(true, &id, &postcond));
        }

        (pre_ts, post_ts)
    }*/
}

// Encodes the specification as a boolean expression for quick checking...
pub trait EncodeQuickCheck {
    fn encode_quick_check(&self) -> (TokenStream, TokenStream);
}

/*impl EncodeQuickCheck for Vec<Specification> {
    fn encode_quick_check(&self, tokens: &mut TokenStream) {
        for spec in self {
            spec.encode_quick_check(tokens);
        }
    }
}

impl EncodeQuickCheck for Specification {
    fn encode_quick_check(&self, tokens: &mut TokenStream) {
        self.assertion.encode_quick_check(tokens);
    }
}*/

/*impl EncodeQuickCheck for TriggerSet {
    fn encode_quick_check(&self, tokens: &mut TokenStream) {
        for trigger_tuple in &self.0 {
            for trigger in &trigger_tuple.0 {
                let span = trigger.expr.span();
                let expr = &trigger.expr;
                let identifier = format!("{}_{}", trigger.spec_id, trigger.id);
                let typeck_call = quote_spanned! { span =>
                    #[prusti::spec_only]
                    #[prusti::expr_id = #identifier]
                    || {
                        #expr
                    };
                };
                tokens.extend(typeck_call);
            }
        }
    }
}*/
impl<EI: Debug> EncodeQuickCheck for common::Assertion<EI, syn::Expr, Arg> {
    fn encode_quick_check(&self) -> (TokenStream, TokenStream) {
        match &*self.kind {
            common::AssertionKind::<EI, syn::Expr, Arg>::Expr(expression) => {
                expression.encode_quick_check()
            }
            common::AssertionKind::<EI, syn::Expr, Arg>::And(assertions) => {
                let (materializations, assertions): (Vec<TokenStream>, Vec<TokenStream>) =
                    assertions
                        .iter()
                        .map(|assert| assert.encode_quick_check())
                        .unzip();
                (
                    quote::quote! {
                        #(#materializations)*
                    },
                    quote::quote! {
                        #(#assertions)&&*
                    },
                )
            }
            common::AssertionKind::<EI, syn::Expr, Arg>::Implies(lhs, rhs) => {
                let (mut lhs_mats, lhs_ts) = lhs.encode_quick_check();
                let (rhs_mats, rhs_ts) = rhs.encode_quick_check();
                lhs_mats.extend(rhs_mats);
                (
                    lhs_mats,
                    quote::quote! {
                        !(#lhs_ts) || (#rhs_ts)
                    },
                )
            }
            /*AssertionKind::ForAll(vars, triggers, body)
            | AssertionKind::Exists(vars, triggers, body) => {
                let vec_of_vars = &vars.vars;
                let span = Span::call_site();
                let identifier = format!("{}_{}", vars.spec_id, vars.id);

                let mut nested_assertion = TokenStream::new();
                body.encode_type_check(&mut nested_assertion);
                triggers.encode_type_check(&mut nested_assertion);

                let typeck_call = quote_spanned! {span=>
                    #[prusti::spec_only]
                    #[prusti::expr_id = #identifier]
                    |#(#vec_of_vars),*| {
                        #nested_assertion
                    };
                };
                tokens.extend(typeck_call);
            }
            AssertionKind::SpecEntailment {closure, arg_binders, pres, posts} => {
                // cl needs special handling because it's not a boolean expression
                let span = closure.expr.span();
                let expr = &closure.expr;
                let cl_id = format!("{}_{}", closure.spec_id, closure.id);
                let typeck_call_cl = quote_spanned! { span =>
                    #[prusti::spec_only]
                    #[prusti::expr_id = #cl_id]
                    || {
                        #expr
                    };
                };
                tokens.extend(typeck_call_cl);

                let span = Span::call_site();
                let pre_id = format!("{}_{}", arg_binders.spec_id, arg_binders.pre_id);
                let post_id = format!("{}_{}", arg_binders.spec_id, arg_binders.post_id);

                let vec_of_args = &arg_binders.args;
                let vec_of_args_with_result: Vec<_> =
                    arg_binders.args
                        .clone()
                        .into_iter()
                        .chain(std::iter::once(arg_binders.result.clone()))
                        .collect();

                let mut pre_assertion = TokenStream::new();
                for pre in pres {
                    pre.encode_type_check(&mut pre_assertion);
                }
                let mut post_assertion = TokenStream::new();
                for post in posts {
                    post.encode_type_check(&mut post_assertion);
                }

                let typeck_call = quote_spanned! { span =>
                    #[prusti::spec_only]
                    #[prusti::expr_id = #pre_id]
                    |#(#vec_of_args),*| {
                        #pre_assertion
                    };
                    #[prusti::spec_only]
                    #[prusti::expr_id = #post_id]
                    |#(#vec_of_args_with_result),*| {
                        #post_assertion
                    };
                };
                tokens.extend(typeck_call);
            }*/
            x => {
                unimplemented!("Havent implemented assertion kind: {:?}", x);
            }
        }
    }
}

impl<EI> EncodeQuickCheck for common::Expression<EI, syn::Expr> {
    fn encode_quick_check(&self) -> (TokenStream, TokenStream) {
        let span = self.expr.span();
        //let expr = &self.expr;
        //let identifier = format!("{}_{}", self.spec_id, self.id);
        // TODO: modify this...
        let (materializations, expr) = rewrite_expr(&self.expr);
        let mat_stream = quote_spanned! { span =>
            #(#materializations)*
        };
        let typeck_call = quote_spanned! { span =>
            #expr
        };
        (mat_stream, typeck_call)
    }
}

// TODO: what do I want to do
// 1. verifier predicates => delete them.
//      - trace_safe and cfg_safe ... just hack it for now...

// This is pretty gross, but handles some wacky cases.
fn rewrite_expr(expr: &syn::Expr) -> (Vec<TokenStream>, syn::Expr) {
    let mut result = expr.clone();
    let mut materializations = vec![];
    // ignore all effects/trace things... lazy way to do it but meh
    if expr_contains_string(expr, "effect") {
        return (materializations, syn::Expr::Verbatim(quote::quote! {true}));
    };
    println!("{:?}", quote::quote! {#expr}.to_string());
    match &mut result {
        Expr::Call(ref mut call) => {
            if func_ident_is(call, "trace_safe") {
                use quote::quote;
                return (materializations, syn::Expr::Verbatim(quote! {true}));
            }
            filter_args(&mut call.args);
            // arg is "old", push it onto materializations stack
            if func_ident_is(call, "old") {
                assert!(call.args.len() <= 1, "Expected old to take only one arg!");
                // we might get None here if we called "old(trace)". We don't want anything
                // with trace so this is fine.
                match call.args.first().clone() {
                    Some(arg) => {
                        let old_id = expr_to_old_id(arg);
                        let ident = syn::Ident::new(old_id.as_str(), Span::call_site());
                        let materialization = quote::quote! {
                            let #ident = #arg;
                        };
                        materializations.push(materialization);
                        // replace old(arg) in expr with materialized id
                        let mut path: Punctuated<syn::PathSegment, syn::token::Colon2> =
                            Punctuated::new();
                        path.push(syn::PathSegment {
                            ident: syn::Ident::new(old_id.as_str(), arg.span()),
                            arguments: syn::PathArguments::None,
                        });
                        let path = syn::Path {
                            leading_colon: None,
                            segments: path,
                        };
                        // replace old(foo) with
                        result = Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path,
                        });
                    }
                    None => {}
                }
            }
        }
        Expr::MethodCall(ref mut method) => {
            filter_args(&mut method.args);
        }
        Expr::Binary(ref mut bin) => {
            let (left_mats, left_expr) = rewrite_expr(bin.left.as_ref());
            let (right_mats, right_expr) = rewrite_expr(bin.right.as_ref());
            bin.left = Box::new(left_expr);
            bin.right = Box::new(right_expr);
            materializations.extend(left_mats);
            materializations.extend(right_mats);
        }
        //_ => unimplemented!("Got unknown type: {:?}", expr),
        _ => {}
    };
    //println!("{:?}", result);
    (materializations, result)
}

fn func_ident_is(call: &syn::ExprCall, keyword: &str) -> bool {
    //match call.
    match &*call.func {
        Expr::Path(path) => path.path.is_ident(keyword),
        _ => unimplemented!("Func name must be simple path!"),
    }
}

fn expr_contains_string(expr: &syn::Expr, keyword: &str) -> bool {
    let tokens = quote::quote! {#expr};
    tokens.to_string().contains(keyword)
}

fn expr_to_old_id(expr: &syn::Expr) -> String {
    let tokens = quote::quote! {#expr};
    let mut res = tokens.to_string().replace("()", "method").replace(" ", "_").replace(".", "dot");
    res.push_str("_old");
    res
}

fn filter_args(args: &mut Punctuated<Expr, syn::token::Comma>) {
    *args = args
        .iter()
        .filter_map(|expr| {
            // TODO: clean this up...
            let path = match expr {
                Expr::Path(path) => path,
                Expr::Field(field) => return Some(expr.clone()),
                _ => unimplemented!("Call arguments must be simple paths! {:?}", expr),
            };
            if path.path.is_ident("trace") {
              None
            } else {
              Some(expr.clone())
            }
        })
        .collect();
}
