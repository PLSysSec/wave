use crate::types::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::multi::{many0, separated_list0};
use nom::sequence::pair;
use nom::{
    bytes::complete::is_not,
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    sequence::delimited,
    IResult,
};
use std::fs;
use std::io;
use std::str::FromStr;
use std::collections::HashMap;

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('), is_not(")"), char(')'))(input)
}

fn curly_brackets(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), is_not("}"), char('}'))(input)
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn qualifier_1arg(input: &str) -> IResult<&str, TypeQualifier> {
    let (input, name) = ws(identifier)(input)?;
    let (_, input) = parens(input)?;
    let (input, arg) = identifier(input)?;
    Ok((input, TypeQualifier::Qualifier1Arg(name.into(), arg.into())))
}

fn qualifier_0arg(input: &str) -> IResult<&str, TypeQualifier> {
    let (input, name) = ws(identifier)(input)?;
    Ok((input, TypeQualifier::Qualifier0Arg(name.into())))
}

fn qualifier(input: &str) -> IResult<&str, TypeQualifier> {
    alt((qualifier_1arg, qualifier_0arg))(input)
}

fn annotation(input: &str) -> IResult<&str, (String, TypeQualifier)> {
    let (input, ty) = ws(identifier)(input)?;
    let (input, _) = ws(tag("="))(input)?;
    let (input, arg_qualifier) = ws(qualifier)(input)?;
    Ok((input, (ty.into(), arg_qualifier)))
}

fn parse_wrapper_policy(input: &str) -> IResult<&str, WrapperPolicy> {
    let (input, _) = tag("Policy")(input)?;
    let (input, function_name) = parens(input)?;
    let (input, _) = ws(tag("="))(input)?;
    let (_, input) = ws(curly_brackets)(input)?;
    let (_, annotations) = separated_list0(tag(","), annotation)(input)?;
    return Ok((
        "",
        WrapperPolicy {
            function_name: function_name.into(),
            annotations,
        },
    ));
}

fn ctype(input: &str) -> IResult<&str, Ctype> {
    let (input, maybe_const) = opt(ws(tag("const")))(input)?;
    let (input, maybe_struct) = opt(ws(tag("struct")))(input)?;
    let (input, ty) = ws(identifier)(input)?;
    let (input, ptr_depth) = many0_count(ws(tag("*")))(input)?;
    let cty = if maybe_struct.is_some(){
        Ctype::CStruct(ty.to_string())
    }
    else{
        Ctype::from_str(ty).unwrap()
    };

    match ptr_depth {
        0 => Ok((input, cty)),
        1 => Ok((input, Ctype::Pointer(Box::new(cty), maybe_const.is_none()))),
        _ => panic!("Unsupported Ctype"),
    }
}

fn arg(input: &str) -> IResult<&str, (Ctype, String)> {
    let (input, ty) = ws(ctype)(input)?;
    let (input, arg_name) = ws(identifier)(input)?;
    Ok((input, (ty, arg_name.into())))
}

fn parse_wrapper_sig(input: &str) -> IResult<&str, WrapperSignature> {
    let (input, ret_ty) = ws(ctype)(input)?;
    let (input, function_name) = ws(identifier)(input)?;
    let (_, args) = parens(input)?;
    let (_, parsed_args) = separated_list0(tag(","), arg)(args)?;
    return Ok((
        "",
        WrapperSignature {
            function_name: function_name.into(),
            ret_ty: ret_ty,
            args: parsed_args,
        },
    ));
}

pub fn parse_spec_from_file(spec_path: String) -> io::Result<Spec> {
    let spec_str = fs::read_to_string(spec_path)?;
    return parse_spec_from_string(spec_str);
}

pub fn parse_spec_from_string(spec_str: String) -> io::Result<Spec> {
    let mut sigs: HashMap<String, WrapperSignature> = HashMap::new();
    let mut policies: HashMap<String, WrapperPolicy> = HashMap::new();

    for line in spec_str.split("\n") {
        println!("{:?}", line);
        if line == "" {
            continue;
        }
        if line.starts_with("Policy") {
            let (_, policy) = parse_wrapper_policy(line).unwrap();
            let fname = policy.function_name.clone();
            policies.insert(fname, policy);
        } else {
            let (_, sig) = parse_wrapper_sig(line).unwrap();
            let fname = sig.function_name.clone();
            sigs.insert(fname, sig);
        }
    }

    return Ok(Spec {
        sigs: sigs,
        policies: policies,
    });
}
