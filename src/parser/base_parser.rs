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

pub use crate::parser::base_parser::Language::Word;
pub use crate::parser::base_parser::Language::Var;
pub use crate::parser::base_parser::Language::Tri;
pub use crate::parser::base_parser::Language::Comp;
pub use crate::parser::base_parser::Triplet::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Language {
    Var(String),
    Get,
    Connector,
    Word(String),
    Tri(Triplet),
    Comp(String),
    Empty
}

#[derive(Clone, PartialEq, Debug)]
pub enum Triplet {
    Twww(String, String, String),
    Tvww(String, String, String),
    Twvw(String, String, String),
    Twwv(String, String, String),
    Tvvw(String, String, String),
    Tvwv(String, String, String),
    Twvv(String, String, String),
    Tvvv(String, String, String)
}

fn to_var(s: &str) -> String {
    format!("${}", s)
}

fn format_tuple_of_three(t: (&String, &String, &String)) -> (String, String, String) {
    match t {
        (a, b, c) => (a.to_owned(), b.to_owned(), c.to_owned()) 
    } 
}

impl Triplet {
    fn to_tuple(&self) -> (String, String, String) {
        match self {
            Twww(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvww(a,b,c) => format_tuple_of_three((a,b,c)),
            Twvw(a,b,c) => format_tuple_of_three((a,b,c)),
            Twwv(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvvw(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvwv(a,b,c) => format_tuple_of_three((a,b,c)),
            Twvv(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvvv(a,b,c) => format_tuple_of_three((a,b,c))
        }
    }
    pub fn to_tuple_with_variable(&self) -> (String, String, String) {
        match self {
            Twww(a,b,c) => (a.to_string(),b.to_string(),c.to_string()),
            Tvww(a,b,c) => (to_var(&a),b.to_string(),c.to_string()),
            Twvw(a,b,c) => (a.to_string(),to_var(&b),c.to_string()),
            Twwv(a,b,c) => (a.to_string(),b.to_string(), to_var(&c)),
            Tvvw(a,b,c) => (to_var(&a),to_var(&b),c.to_string()),
            Tvwv(a,b,c) => (to_var(&a),b.to_string(),to_var(&c)),
            Twvv(a,b,c) => (a.to_string(),to_var(&b),to_var(&c)),
            Tvvv(a,b,c) => (to_var(&a),to_var(&b),to_var(&c))
        }
    }
}

fn parse_variable_or_star(s: &str) -> IResult<&str, Language> {
    let res = alt((
            preceded(char('$'), alphanumeric1),
            tag("*")
            ))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Var(s.to_string()))),
        Err(e) => Err(e)
    }
}

pub fn parse_variable(s: &str) -> IResult<&str,Language> {
    preceded(
        space1,
        parse_variable_or_star
        )(s)
}

fn parse_word(s: &str) -> IResult<&str,Language> {
    let res = preceded(space1, alphanumeric1)(s);
    match res {
        Ok((t, s)) => Ok((t, Word(s.to_string()))),
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

    #[test]
    fn test_triplet_and() {
        assert_eq!(
            parse_triplet_and(" B ami C AND A ami C").unwrap().1,
            Tri(Twww("B","ami","C"))
        );
    }


}
