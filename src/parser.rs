#![allow(dead_code, unused_variables, unused_imports)]

use nom::{
    bytes::complete::tag,
    character::complete::{char, alphanumeric1},
    sequence::preceded,
    Needed,
    IResult
};

#[derive(PartialEq, Debug)]
enum LanguageType<'a> {
    Soft(&'a str),
    Raw(&'a str),
    SQL(&'a str)
}

#[derive(PartialEq, Debug)]
enum Language<'a> {
    Var(&'a str),
    Get
}

fn parse_variable(s: &str) -> IResult<&str,Language> {
    let res = preceded(char('$'), alphanumeric1)(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Var(s))),
        Err(e) => Err(e)
    }
}

fn parse_get(s: &str) -> IResult<&str,Language> {
    let res = tag("get")(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Get)),
        Err(e) => Err(e)
    }
}

fn parse_query_helper(s: &str) -> IResult<&str,&str> {
    tag("get")(s)
}

fn to_sql(s: &str) -> &str {
    s
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        assert_eq!(parse_get("get").unwrap().1, Language::Get);
        assert_eq!(
            parse_get("select"),
            Err(nom::Err::Error(
                    nom::error::Error {
                        input: "select",
                        code: nom::error::ErrorKind::Tag})));
    }

    #[test]
    fn test_variable() {
        assert_eq!(
            parse_variable("$Hello").unwrap().1,
            Language::Var("Hello"));
        assert_eq!(
            parse_variable("$A").unwrap().1,
            Language::Var("A"));
        assert_eq!(
            parse_variable("$2").unwrap().1,
            Language::Var("2"));
        assert_eq!(
            parse_variable("hey"),
            Err(nom::Err::Error(
                nom::error::Error {
                    input: "hey",
                    code: nom::error::ErrorKind::Char
                }
            )));
    } 

    #[test]
    fn test_final(){
        assert_eq!(2,2);
        //assert_eq!(
    //parse_query(LanguageType::Soft("get $A $B | $A age $B where $B < 30;")),
    //LanguageType::SQL("select A, B from (select subject as A, goal as B from facts where link like 'age') where B < 30);")
            //);
    }
}
