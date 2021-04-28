use std::fs;
use std::io;
use nom::{
    IResult,
    sequence::delimited,
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::{char, multispace0, alphanumeric1, alpha1},
    bytes::complete::is_not
};
use nom::error::ParseError;
use nom::branch::alt;
use nom::multi::{many0, separated_list0};
use nom::combinator::recognize;
use nom::sequence::pair;
use nom::bytes::complete::tag;



#[derive(Debug)]
pub struct WrapperSignature {}

#[derive(Debug)]
pub struct WrapperPolicy {
    function_name: String,
    annotations: Vec<(String, TypeQualifier)>,
}

#[derive(Debug)]
pub struct Spec {
    sigs: Vec<WrapperSignature>,
    policies: Vec<WrapperPolicy>
}

#[derive(Debug)]
pub enum TypeQualifier {
    Qualifier0Arg(String),
    Qualifier1Arg(String, String),
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(
      pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"))))
      )
    )(input)
}

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('), is_not(")"), char(')'))(input)
}

fn curly_brackets(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), is_not("}"), char('}'))(input)
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
  where
  F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
  delimited(
    multispace0,
    inner,
    multispace0
  )
}

fn qualifier_1arg(input: &str) -> IResult<&str, TypeQualifier> {
    let (input, name) = ws(identifier)(input)?;
    let (input, _) = parens(input)?;
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

fn parse_wrapper_policy(input: &str) -> IResult<&str, WrapperPolicy>{
    let (input, _) = tag("Policy")(input)?;
    let (input, function_name) = parens(input)?;
    let (input, _) = ws(tag("="))(input)?;
    let (input, _) = ws(curly_brackets)(input)?;
    let (input, annotations) = separated_list0(tag(","), annotation)(input)?;
    return Ok( ("",WrapperPolicy{function_name: function_name.into(), annotations})) ;
}

fn parse_wrapper_sig(line: &str) -> WrapperSignature{
    return WrapperSignature {};
}

pub fn parse_spec_from_file(spec_path: String) -> io::Result<Spec>{
    let spec_str = fs::read_to_string("address.txt")?;
    return parse_spec_from_string(spec_str);
}

pub fn parse_spec_from_string(spec_str: String) -> io::Result<Spec>{
    let mut sigs: Vec<WrapperSignature> = Vec::new();
    let mut policies: Vec<WrapperPolicy> = Vec::new();

    for line in spec_str.split("\n"){
        if line.starts_with("Policy"){
            let (_,policy) = parse_wrapper_policy(line).unwrap();
            policies.push(policy);
        }
        else{
            let sig = parse_wrapper_sig(line);
            sigs.push(sig);
        }
    }

    return Ok(Spec {sigs: sigs, policies: policies});
}
