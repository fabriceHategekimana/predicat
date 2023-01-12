#![allow(dead_code, unused_variables, unused_imports)]
use nom::{
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
use crate::parser::Language::Word;
use crate::parser::Language::Var;
use crate::parser::Language::Tri;
use crate::parser::Language::Comp;

use crate::parser::Triplet::*;

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
    Comp(&'a str)
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

fn parse_variable(s: &str) -> IResult<&str,Language> {
    let res = preceded(
        space1,
        preceded(char('$'), alphanumeric1),
        )(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Var(s))),
        Err(e) => Err(e)
    }
}

fn recognize_variable(s: &str) -> IResult<&str,&str> {
    preceded(
        space1,
        recognize(preceded(char('$'), alphanumeric1)),
        )(s)
}

fn parse_get(s: &str) -> IResult<&str,Language> {
    let res = tag("get")(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Get)),
        Err(e) => Err(e)
    }
}

fn parse_connector(s: &str) -> IResult<&str, Language> {
    let res =alt((tag(" such_as"),
        tag(" who_is"),
        tag(" who_are"),
        tag(" who_has")))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Connector)),
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

fn parse_triplet_and(s: &str) -> IResult<&str,Language> {
    alt((
        terminated(parse_triplet, tag(" and")),
        parse_triplet))(s)
}

fn parse_operator(s: &str) -> IResult<&str,&str> {
    preceded(space1, alt((
        tag("=="),
        tag("!="),
        tag("<="),
        tag(">="),
        tag("<"),
        tag(">")
        )))(s)
}

fn parse_comparison(s: &str) -> IResult<&str,Language> {
    let res = recognize(tuple((
                parse_valvar,
                parse_operator,
                parse_valvar
                )))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Comp(s))),
        Err(e) => Err(e)
    }
}

fn parse_comparison_and(s: &str) -> IResult<&str,Language> {
    alt((
        terminated(parse_comparison, tag(" and")),
        parse_comparison))(s)
}

fn parse_number(s: &str) -> IResult<&str,&str> {
   preceded(space1,
      alt((
          recognize(tuple((char('-'),digit1,char('.'),digit1))),
          recognize(tuple((digit1,char('.'),digit1))),
          recognize(tuple((char('-'), digit1))),
          digit1)
       ))(s)
}

fn parse_string(s: &str) -> IResult<&str,&str> {
   recognize(delimited(
       char('\''),
       is_not("\'"),
       char('\'')))(s)
}

fn parse_value(s: &str) -> IResult<&str,&str> {
    alt((parse_string, parse_number))(s)
}

fn parse_valvar(s: &str) -> IResult<&str,&str> {
    alt((recognize_variable, parse_value))(s)
}

fn parse_query_helper(s: &str) -> IResult<&str,&str> {
    tag("get")(s)
}


fn to_sql(s: &str) -> &str {
    todo!();
}

fn parse_and_convert(s: &str) -> &str {
    let res = parse_query_helper(s);
    println!("res: {:?}", res);
    to_sql(res.unwrap().1)
}

fn parse_query(query: LanguageType) -> LanguageType {
    // take a soft query and return an sql query
    match query {
        LanguageType::Soft(q) => LanguageType::SQL(parse_and_convert(q)),
        _ => LanguageType::SQL("sql")
    }
}

pub fn parse_query2(s: &str) -> IResult<&str,(Vec<Language>,Vec<Language>,Vec<Language>)> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_triplet_and),
          many1(parse_comparison_and)))(s);
    match res {
        Ok((r, (g,var,c,tri,comp))) => Ok((r, (var, tri, comp))),
        Err(e) => Err(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        assert_eq!(parse_get("get").unwrap().1, Language::Get);
        assert_eq!(
            parse_get("select"),
            Err(nom::Err::Error(
                    Error {
                        input: "select",
                        code: ErrorKind::Tag})));
    }

    #[test]
    fn test_variable() {
        assert_eq!(
            parse_variable(" $Hello").unwrap().1,
            Language::Var("Hello"));
        assert_eq!(
            parse_variable(" $A").unwrap().1,
            Language::Var("A"));
        assert_eq!(
            parse_variable(" $2").unwrap().1,
            Language::Var("2"));
        assert_eq!(
            parse_variable(" hey"),
            Err(nom::Err::Error(
                Error {
                    input: "hey",
                    code: ErrorKind::Char
                }
            )));
    } 
    #[test]
    fn test_connector() {
        assert_eq!(
            parse_connector(" who_has").unwrap().1,
            Language::Connector);
        assert_eq!(
            parse_connector(" such_as").unwrap().1,
            Language::Connector);
        assert_eq!(
            parse_connector(" hey"),
            Err(nom::Err::Error(
                Error {
                    input: " hey",
                    code: ErrorKind::Tag
                }
            )));
    }
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
    fn test_operator() {
        assert_eq!(
            parse_operator(" ==").unwrap().1,
            "==");
        assert_eq!(
            parse_operator(" >").unwrap().1,
            ">");
        assert_eq!(
            parse_operator(" a"),
            Err(nom::Err::Error(
                Error {
                    input: "a",
                    code: ErrorKind::Tag
                }
            )));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(
            parse_comparison(" 4 < 5").unwrap().1,
            Language::Comp(" 4 < 5"));
        assert_eq!(
            parse_comparison(" $A > 5").unwrap().1,
            Language::Comp(" $A > 5"));
        assert_eq!(
            parse_comparison(" F"),
            Err(
                nom::Err::Error(
                    Error { input: "F", code: ErrorKind::Digit })));
    }


    #[test]
    fn test_string() {
        assert_eq!(
            parse_string("'un deux trois'").unwrap().1,
            "'un deux trois'"
            );
    }

    #[test]
    fn test_number() {
        assert_eq!(
            parse_number(" 57").unwrap().1,
            "57");
        assert_eq!(
            parse_number(" 2.57").unwrap().1,
            "2.57");
        assert_eq!(
            parse_number(" -57").unwrap().1,
            "-57");
        assert_eq!(
            parse_number(" -57.34").unwrap().1,
            "-57.34");
    }

    #[test]
    fn test_valvar() {
        assert_eq!(
            parse_valvar(" $A").unwrap().1,
            "$A");
    }

    #[test]
    fn test_parse_query2() {
        //TODO modify to let the "and" keyword
        assert_eq!(parse_query2("get $A such_as $A ami Bob $A == 7").unwrap().1,
                   (vec![Var("A")], vec![Tri(Tvww("A","ami","Bob"))], vec![Comp(" $A == 7")])
                   );
        assert_eq!(parse_query2("get $A $B such_as $A ami $B $A == 7").unwrap().1,
                   (vec![Var("A"), Var("B")], vec![Tri(Tvwv("A","ami","B"))], vec![Comp(" $A == 7")])
                   );
        assert_eq!(parse_query2("get $A $B such_as $A ami $B and $A == 7 and $B < 9").unwrap().1,
                   (vec![Var("A"), Var("B")], vec![Tri(Tvwv("A","ami","B"))], vec![Comp(" $A == 7"), Comp(" $B < 9")])
                   );
    }

    #[test]
    fn test_triplet_and() {
        assert_eq!(
            parse_triplet_and(" B ami C and A ami C").unwrap().1,
            Tri(Twww("B","ami","C"))
        );
    }

    #[test]
    fn test_comparison_and() {
        assert_eq!(
            parse_comparison_and(" 7 == 8 and 6 < 9").unwrap().1,
            Comp(" 7 == 8"));
    }
}
