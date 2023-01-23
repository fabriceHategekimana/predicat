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
   recognize(preceded(space1,delimited(
       char('\''),
       is_not("\'"),
       char('\''))))(s)
}

fn parse_value(s: &str) -> IResult<&str,&str> {
    alt((parse_string, parse_number))(s)
}

fn parse_valvar(s: &str) -> IResult<&str,&str> {
    alt((recognize_variable, parse_value))(s)
}

fn parse_query2_var1(s: &str) -> IResult<&str,(Vec<Language>, Vec<Language>,Vec<Language>)> {
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

fn parse_query2_var2(s: &str) -> IResult<&str,(Vec<Language>, Vec<Language>,Vec<Language>)> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_triplet_and)))(s);
    match res {
        Ok((r, (g,var,c,tri))) => Ok((r, (var, tri, vec![Language::Empty]))),
        Err(e) => Err(e)
    }
}

fn parse_query2_var3(s: &str) -> IResult<&str,(Vec<Language>, Vec<Language>,Vec<Language>)> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_comparison_and)))(s);
    match res {
        Ok((r, (g,var,c,comp))) => Ok((r, (var, vec![Language::Empty], comp))),
        Err(e) => Err(e)
    }
}
pub fn parse_query2(s: &str) -> IResult<&str,(Vec<Language>,Vec<Language>,Vec<Language>)> {
    alt((
        parse_query2_var1,
        parse_query2_var2,
        parse_query2_var3
        ))(s)
}

fn format_variables(vars: &[Language]) -> String {
    if vars == [Language::Empty]{
        String::from("Select * from ")
    }
    else {
        let extracted_vars = vars.iter()
            .filter_map(|x| {
                match x {
                    Var(v) => Some(v),
                    _ => None
                }
            });
        let string_vars = extracted_vars
            .fold("".to_string(), |acc, &x| acc +","+x)
            .chars()
            .skip(1)
            .collect::<String>();
        format!("Select {} from ",string_vars)
    }
}

fn triplet_to_sql(tri: &Triplet) -> String {
    match tri {
        Twww(a,b,c) => 
            format!("Select subject,link,goal from facts where subject='{}' and link='{}' and goal='{}'",a,b,c),
        Tvww(a,b,c) => 
            format!("Select subject as {} from facts where link='{}' and goal='{}'",a,b,c),
        Twvw(a,b,c) => 
            format!("Select link as {} from facts where subject='{}' and goal='{}'",b,a,c),
        Twwv(a,b,c) => 
            format!("Select goal as {} from facts where subject='{}' and link='{}'",c,a,b),
        Tvvw(a,b,c) => 
            format!("Select subject as {},link as {} from facts where goal='{}'",a,b,c),
        Tvwv(a,b,c) => 
            format!("Select subject as {},goal as {} from facts where link='{}'",a,c,b),
        Twvv(a,b,c) => 
            format!("Select link as {},goal as {} from facts where subject='{}'",b,c,a),
        Tvvv(a,b,c) => 
            format!("Select subject as {},link as {},goal as {} from facts",a,b,c),
    }
}

fn format_triplets(tri: &[Language]) -> String {
    if tri == [Language::Empty]{
        String::from("facts")
    }
    else {
        let sql_queries = tri.iter()
            .filter_map(|x| {
                match x {
                    Tri(t) => Some(triplet_to_sql(t)),
                    _ => None
                }
            });
        let queries = sql_queries
            .reduce(|acc, x| format!("{} natural join {}", acc, x)).unwrap();
        format!("({})", queries)
    }
}

fn format_comparisons(comp: &[Language]) -> String {
    if  comp == [Language::Empty] {
        String::from(";")
    }
    else {
        let comparisons = comp.iter()
            .filter_map(|x| {
                match x {
                    Comp(c) => Some(c.replace("$","").replace("==","=")),
                    _ => None
                }
            });
        let final_comparisons = comparisons
            .reduce(|acc, x| format!("{} and {}", acc, x)).unwrap();
        format!(" where {};", final_comparisons)
    }
}

fn to_sql(res: (&[Language], &[Language], &[Language])) -> String {
    let head = format_variables(&res.0);
    let columns = format_triplets(&res.1);
    let comparisons = format_comparisons(&res.2);
    format!("{}{}{}", head, columns, comparisons )
}

pub fn parse_query(s: &str) -> String {
    let res = parse_query2(s);
    match res {
        Ok((t, (v1,v2,v3))) => to_sql((&v1,&v2,&v3)),
        Err(e) => format!("{}", e)
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
        assert_eq!(
            parse_comparison(" 4 == 5").unwrap().1,
            Language::Comp(" 4 == 5"));
        assert_eq!(
            parse_comparison(" 4 == 'res'").unwrap().1,
            Language::Comp(" 4 == 'res'"));
    }

    #[test]
    fn test_string() {
        assert_eq!(
            parse_string(" 'un deux trois'").unwrap().1,
            " 'un deux trois'"
            );
        assert_eq!(
            parse_string(" 'sdt'").unwrap().1,
            " 'sdt'"
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
        assert_eq!(
            parse_valvar(" 7").unwrap().1,
            "7");
        assert_eq!(
            parse_valvar(" '7'").unwrap().1,
            " '7'");
    }

    #[test]
    fn test_parse_query2() {
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
    #[test]
    fn test_format_variables() {
        assert_eq!(
            format_variables(&vec![Var("X"),Var("Y")]),
            "Select X,Y from ("
        );
        assert_eq!(
            format_variables(&vec![Var("X")]),
            "Select X from ("
        );
    }
    #[test]
    fn test_from_triplet_to_sql() {
        assert_eq!(
            triplet_to_sql(&Tvvv("A","B","C")),
            "Select subject as A,link as B,goal as C from facts".to_string()
        );
        assert_eq!(
            triplet_to_sql(&Tvwv("A","B","C")),
            "Select subject as A,goal as C from facts where link='B'"
        );
    }

    #[test]
    fn test_format_triplets() {
        assert_eq!(
            format_triplets(&vec![Tri(Tvvv("A","B","C"))]),
            "Select subject as A,link as B,goal as C from facts".to_string()
        );
        assert_eq!(
            format_triplets(&vec![Tri(Tvvv("A","B","C")),Tri(Twvv("D","E","F"))]),
            "Select subject as A,link as B,goal as C from facts natural join Select link as E,goal as F from facts where subject='D'".to_string()
        );
    }

    #[test]
    fn test_format_comparisons() {
        assert_eq!(
            format_comparisons(&vec![Comp(" $A == 8")]),
            ") where  A = 8;".to_string()
        );
        assert_eq!(
            format_comparisons(&vec![Comp(" $A == 8"), Comp(" 6 < 3")]),
            ") where  A = 8 and  6 < 3;".to_string()
        );
    }

    #[test]
    fn test_value() {
        assert_eq!(
            parse_value(" -57.34").unwrap().1,
            "-57.34");

        assert_eq!(
            parse_value(" '3'").unwrap().1,
            " '3'"
            );

        assert_eq!(
            parse_value(" 'sdt'").unwrap().1,
            " 'sdt'"
            );
    }
}
