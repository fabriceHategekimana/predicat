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
    fn to_tuple_with_variable(&self) -> (String, String, String) {
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


