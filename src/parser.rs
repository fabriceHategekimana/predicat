#![allow(dead_code, unused_variables, unused_imports)]

use nom::{
    bytes::complete::tag,
    character::complete::{char, alphanumeric1, space1},
    sequence::{preceded, tuple},
    branch::alt,
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
    Get,
    Connector,
    Word(&'a str),
    Triplet(&'a str, &'a str, &'a str)
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

fn parse_get(s: &str) -> IResult<&str,Language> {
    let res = tag("get")(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Get)),
        Err(e) => Err(e)
    }
}

fn parse_connector(s: &str) -> IResult<&str, Language> {
    let res =alt((tag("such_as"),
        tag("who_is"),
        tag("who_are"),
        tag("who_has")))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Connector)),
        Err(e) => Err(e)
    }
}

fn parse_word(s: &str) -> IResult<&str,Language> {
    let res = preceded(space1, alphanumeric1)(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Word(s))),
        Err(e) => Err(e)
    }
}

fn parse_triplet(s: &str) -> IResult<&str,Language> {
    let res = alt((
            tuple((parse_word, parse_word, parse_word)),
            tuple((parse_word, parse_word, parse_variable))
            ))(s);
    match res {
        Ok((t, tri)) => Ok((t, to_triplet(tri))) ,
        Err(e) => Err(e),
        _ => todo!()
    }
}

fn parse_word_space(s: &str) -> IResult<&str, Language> {
    let res = preceded(space1, alphanumeric1)(s);
    match res {
        Ok((t, s1)) => Ok((t, Language::Word(s1))),
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

fn extract(l: Language) -> Option<&str> {
    match l {
        Language::Var(s) => Some(s),
        Language::Word(s) => Some(s),
        _ => None
    }
}

fn to_triplet<'a>(t: (Language<'a>, Language<'a>, Language<'a>)) -> Language<'a> {
    Language::Triplet(
        extract(t.0).unwrap(),
        extract(t.1).unwrap(),
        extract(t.2).unwrap()
        )
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
                nom::error::Error {
                    input: "hey",
                    code: nom::error::ErrorKind::Char
                }
            )));
    } 
    #[test]
    fn test_connector() {
        assert_eq!(
            parse_connector("who_has").unwrap().1,
            Language::Connector);
        assert_eq!(
            parse_connector("such_as").unwrap().1,
            Language::Connector);
        assert_eq!(
            parse_connector("hey"),
            Err(nom::Err::Error(
                nom::error::Error {
                    input: "hey",
                    code: nom::error::ErrorKind::Tag
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
                nom::error::Error {
                    input: "$A",
                    code: nom::error::ErrorKind::AlphaNumeric
                }
            )));
    }
    #[test]
    fn test_triplet() {
        assert_eq!(
            parse_triplet(" un deux trois").unwrap().1,
            Language::Triplet("un", "deux", "trois"));
        assert_eq!(
            parse_triplet(" un deux $A").unwrap().1,
            Language::Triplet("un", "deux", "A"));
    }
    #[test]
    fn test() {
        assert_eq!(
            parse_word_space(" hey").unwrap().1,
            Language::Word("hey"));
    }
}
