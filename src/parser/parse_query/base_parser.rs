pub use nom::{
    bytes::complete::{tag, is_not},
    character::complete::{char, alphanumeric1, space1, digit1},
    sequence::{preceded, tuple, delimited, terminated},
    branch::alt,
    combinator::recognize,
    multi::many1,
    error::{Error,
            ErrorKind},
    IResult
};


//for ease of use
pub use crate::parser::parse_query::base_parser::Language::Word;
pub use crate::parser::parse_query::base_parser::Language::Var;
pub use crate::parser::parse_query::base_parser::Language::Tri;
pub use crate::parser::parse_query::base_parser::Language::Comp;
pub use crate::parser::parse_query::base_parser::Triplet::*;

#[derive(PartialEq, Debug)]
enum LanguageType<'a> {
    Soft(&'a str),
    Raw(&'a str),
    SQL(&'a str)
}

#[derive(PartialEq, Debug)]
pub enum Language<'a> {
    Var(&'a str),
    Get,
    Connector,
    Word(&'a str),
    Tri(Triplet<'a>),
    Comp(&'a str),
    Empty
}

#[derive(PartialEq, Debug)]
pub enum Triplet<'a> {
    Twww(&'a str, &'a str, &'a str),
    Tvww(&'a str, &'a str, &'a str),
    Twvw(&'a str, &'a str, &'a str),
    Twwv(&'a str, &'a str, &'a str),
    Tvvw(&'a str, &'a str, &'a str),
    Tvwv(&'a str, &'a str, &'a str),
    Twvv(&'a str, &'a str, &'a str),
    Tvvv(&'a str, &'a str, &'a str)
}

fn to_var(s: &str) -> String {
    format!("${}", s)
}

impl <'a>Triplet<'a> {
    fn to_tuple(&self) -> (&'a str, &'a str, &'a str) {
        match *self {
            Twww(a,b,c) => (a,b,c),
            Tvww(a,b,c) => (a,b,c),
            Twvw(a,b,c) => (a,b,c),
            Twwv(a,b,c) => (a,b,c),
            Tvvw(a,b,c) => (a,b,c),
            Tvwv(a,b,c) => (a,b,c),
            Twvv(a,b,c) => (a,b,c),
            Tvvv(a,b,c) => (a,b,c)
        }
    }
    pub fn to_tuple_with_variable(&self) -> (String, String, String) {
        match *self {
            Twww(a,b,c) => (a.to_string(),b.to_string(),c.to_string()),
            Tvww(a,b,c) => (to_var(a),b.to_string(),c.to_string()),
            Twvw(a,b,c) => (a.to_string(),to_var(b),c.to_string()),
            Twwv(a,b,c) => (a.to_string(),b.to_string(), to_var(c)),
            Tvvw(a,b,c) => (to_var(a),to_var(b),c.to_string()),
            Tvwv(a,b,c) => (to_var(a),b.to_string(),to_var(c)),
            Twvv(a,b,c) => (a.to_string(),to_var(b),to_var(c)),
            Tvvv(a,b,c) => (to_var(a),to_var(b),to_var(c))
        }
    }
}

pub fn parse_variable(s: &str) -> IResult<&str,Language> {
    let res = preceded(
        space1,
        preceded(char('$'), alphanumeric1),
        )(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Var(s))),
        Err(e) => Err(e)
    }
}

fn parse_word(s: &str) -> IResult<&str,Language> {
    let res = preceded(space1, alphanumeric1)(s);
    match res {
        Ok((t, s)) => Ok((t, Word(s))),
        Err(e) => Err(e)
    }
}

fn parse_triplet(s: &str) -> IResult<&str,Language> {
    let res = alt((
            tuple((parse_word, parse_word, parse_word)),
            tuple((parse_variable, parse_word, parse_word)),
            tuple((parse_word, parse_variable, parse_word)),
            tuple((parse_word, parse_word, parse_variable)),
            tuple((parse_variable, parse_variable, parse_word)),
            tuple((parse_variable, parse_word, parse_variable)),
            tuple((parse_word, parse_variable, parse_variable)),
            tuple((parse_variable, parse_variable, parse_variable))
            ))(s);
    match res {
        Ok((t, (Word(s1),Word(s2),Word(s3)))) => Ok((t, Tri(Twww(s1,s2,s3)))),
        Ok((t, (Var(s1),Word(s2),Word(s3)))) => Ok((t, Tri(Tvww(s1,s2,s3)))),
        Ok((t, (Word(s1),Var(s2),Word(s3)))) => Ok((t, Tri(Twvw(s1,s2,s3)))),
        Ok((t, (Word(s1),Word(s2),Var(s3)))) => Ok((t, Tri(Twwv(s1,s2,s3)))),
        Ok((t, (Var(s1),Var(s2),Word(s3)))) => Ok((t, Tri(Tvvw(s1,s2,s3)))),
        Ok((t, (Var(s1),Word(s2),Var(s3)))) => Ok((t, Tri(Tvwv(s1,s2,s3)))),
        Ok((t, (Word(s1),Var(s2),Var(s3)))) => Ok((t, Tri(Twvv(s1,s2,s3)))),
        Ok((t, (Var(s1),Var(s2),Var(s3)))) => Ok((t, Tri(Tvvv(s1,s2,s3)))),
        Err(e) => Err(e),
        _ => todo!()
    }
}

pub fn parse_triplet_and(s: &str) -> IResult<&str,Language> {
    alt((
        terminated(parse_triplet, tag(" and")),
        parse_triplet))(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word() {
        assert_eq!(
            parse_word(" wow").unwrap().1,
            Language::Word("wow"));
        assert_eq!(
            parse_word(" $A"),
            Err(nom::Err::Error(
                Error {
                    input: "$A",
                    code: ErrorKind::AlphaNumeric
                }
            )));
    }

    #[test]
    fn test_triplet() {
        assert_eq!(
            parse_triplet(" un deux trois").unwrap().1,
            Language::Tri(Twww("un", "deux", "trois")));
        assert_eq!(
            parse_triplet(" un deux $A").unwrap().1,
            Language::Tri(Twwv("un", "deux", "A")));
        assert_eq!(
            parse_triplet(" $A deux trois").unwrap().1,
            Language::Tri(Tvww("A", "deux", "trois")));
    }

}
