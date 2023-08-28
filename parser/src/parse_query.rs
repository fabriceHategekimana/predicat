pub use nom::{
    bytes::complete::{tag, is_not},
    character::complete::{char, alphanumeric1, space1, digit1},
    sequence::{preceded, tuple, delimited, terminated},
    branch::alt,
    combinator::recognize,
    multi::many1,
    IResult,
};

pub use super::base_parser::{
    Language,
    Var,
    Comp,
    Triplet,
    Triplet::*,
    parse_variable,
    parse_triplet_and,
    extract_triplet
};

use nom::Err;
use nom::Needed;
use super::base_parser::PredicatAST;

type QueryAST = (Vec<Var>, Vec<Triplet>,Vec<Comp>);
type QueryVarAST<'a> = ((Vec<Language>, Vec<Language>,Vec<Language>), Vec<&'a str>);

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

fn recognize_variable(s: &str) -> IResult<&str,&str> {
    preceded(
        space1,
        recognize(preceded(char('$'), alphanumeric1)),
        )(s)
}

fn parse_string(s: &str) -> IResult<&str,&str> {
   recognize(preceded(space1,delimited(
       char('\''),
       is_not("\'"),
       char('\''))))(s)
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

fn parse_value(s: &str) -> IResult<&str,&str> {
    alt((parse_string, parse_number))(s)
}

fn parse_valvar(s: &str) -> IResult<&str,&str> {
    alt((recognize_variable, parse_value))(s)
}


fn parse_comparison_and(s: &str) -> IResult<&str,Language> {
    alt((
        terminated(parse_comparison, tag(" and")),
        parse_comparison))(s)
}

fn parse_comparison(s: &str) -> IResult<&str,Language> {
    let res = recognize(tuple((
                parse_valvar,
                parse_operator,
                parse_valvar
                )))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Comp(s.to_string()))),
        Err(e) => Err(e)
    }
}

fn parse_connector(s: &str) -> IResult<&str, Language> {
    let res =alt((tag(" such_as"),
        tag(" who_is"),
        tag(" who_are"),
        tag(" who_has"),
        tag(" where")))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Connector)),
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


// get [vars] [connector] [triplets] [comparison]
fn parse_query_var1(s: &str) -> IResult<&str, QueryAST> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_triplet_and),
          many1(parse_comparison_and)))(s);
    match res {
        Ok((r, (g, var, c, vtri, comp))) => Ok((r, 
                        (var.iter().flat_map(Language::get_var).collect(),
                        vtri.iter().flat_map(extract_triplet).collect(),
                        comp.iter().flat_map(Language::get_comp).collect()))),
        Err(e) => Err(e)
    }
}

// get [variables] [connector] [triplets]
fn parse_query_var2(s: &str) -> IResult<&str, QueryAST> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_triplet_and)))(s);
    match res {
        Ok((r, (g,var,c,vtri))) => Ok((r, (var.iter().flat_map(Language::get_var).collect()
                                           , vtri.iter().flat_map(extract_triplet).collect(), 
                                           vec![]))),
        Err(e) => Err(e)
    }
}

// get [vars] [connector] [comparison]
fn parse_query_var3(s: &str) -> IResult<&str, QueryAST> {
    let res = tuple((parse_get,
          many1(parse_variable),
          parse_connector,
          many1(parse_comparison_and)))(s);
    match res {
        Ok((r, (g,var,c,comp))) => Ok((r, (var.iter().flat_map(Language::get_var).collect(),
                                           vec![Triplet::Empty], 
                                           comp.iter().flat_map(Language::get_comp).collect()))),
        Err(e) => Err(e)
    }
}

pub fn parse_query(s: &str) -> IResult<&str, PredicatAST> {
    let res = alt((
        parse_query_var1,
        parse_query_var2,
        parse_query_var3
        ))(s);
    match res {
        Ok((s, (var, tri, comp))) => Ok((s, PredicatAST::Query((var, tri, comp)))),
        Err(e) => Err(e)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_modifier::Triplet::*, parse_query::recognize_variable};
    use nom::error::{Error, ErrorKind};
    use crate::PredicatAST::Query;
    use super::{
        Language,
        Var,
        Comp,
        Triplet,
        PredicatAST,
        parse_get,
        parse_variable,
        parse_connector,
        parse_operator,
        parse_comparison,
        parse_string,
        parse_number,
        parse_valvar,
        parse_comparison_and,
        parse_value,
        parse_query_var1,
        parse_query_var2,
        parse_query_var3,
        parse_query,
    };

    #[test]
    fn test_get() {
        assert_eq!(parse_get("get").unwrap().1, Language::Get);
        assert_eq!(
            parse_get("SELECT"),
            Err(nom::Err::Error(
                    Error {
                        input: "SELECT",
                        code: ErrorKind::Tag})));
    }

    #[test]
    fn test_variable() {
        assert_eq!(
            parse_variable(" $Hello").unwrap().1,
            Language::Var("Hello".to_string()));
        assert_eq!(
            parse_variable(" $A").unwrap().1,
            Language::Var("A".to_string()));
        assert_eq!(
            parse_variable(" $2").unwrap().1,
            Language::Var("2".to_string()));
        assert_eq!(
            parse_variable(" hey"),
            Err(nom::Err::Error(
                Error {
                    input: "hey",
                    code: ErrorKind::Tag
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
    fn test_operator() {
        assert_eq!(
            parse_operator(" ==").unwrap().1,
            "==".to_string());
        assert_eq!(
            parse_operator(" >").unwrap().1,
            ">".to_string());
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
            Language::Comp(" 4 < 5".to_string()));
        assert_eq!(
            parse_comparison(" $A > 5").unwrap().1,
            Language::Comp(" $A > 5".to_string()));
        assert_eq!(
            parse_comparison(" F"),
            Err(
                nom::Err::Error(
                    Error { input: "F", code: ErrorKind::Digit })));
        assert_eq!(
            parse_comparison(" 4 == 5").unwrap().1,
            Language::Comp(" 4 == 5".to_string()));
        assert_eq!(
            parse_comparison(" 4 == 'res'").unwrap().1,
            Language::Comp(" 4 == 'res'".to_string()));
    }

    #[test]
    fn test_comparison2() {
        assert_eq!(
            parse_comparison(" 4 < 5 | ").unwrap(),
            (" | ", Language::Comp(" 4 < 5".to_string())));
    }

    #[test]
    fn test_string() {
        assert_eq!(
            parse_string(" 'un deux trois'").unwrap().1,
            " 'un deux trois'".to_string()
            );
        assert_eq!(
            parse_string(" 'sdt'").unwrap().1,
            " 'sdt'".to_string()
            );
    }

    #[test]
    fn test_number() {
        assert_eq!(
            parse_number(" 57").unwrap().1,
            "57".to_string());
        assert_eq!(
            parse_number(" 2.57").unwrap().1,
            "2.57".to_string());
        assert_eq!(
            parse_number(" -57").unwrap().1,
            "-57".to_string());
        assert_eq!(
            parse_number(" -57.34").unwrap().1,
            "-57.34".to_string());
    }

    #[test]
    fn test_valvar() {
        assert_eq!(
            parse_valvar(" $A").unwrap().1,
            "$A".to_string());
        assert_eq!(
            parse_valvar(" 7").unwrap().1,
            "7".to_string());
        assert_eq!(
            parse_valvar(" '7'").unwrap().1,
            " '7'".to_string());
    }

    #[test]
    fn test_comparison_and() {
        assert_eq!(
            parse_comparison_and(" 7 == 8 AND 6 < 9").unwrap().1,
            Language::Comp(" 7 == 8".to_string()));
    }

    #[test]
    fn test_comparison_and2() {
        assert_eq!(
            parse_comparison_and(" 7 == 8 AND 6 < 9 | ").unwrap(),
            (" AND 6 < 9 | ", Language::Comp(" 7 == 8".to_string())));
    }

    #[test]
    fn test_value() {
        assert_eq!(
            parse_value(" -57.34").unwrap().1,
            "-57.34".to_string());

        assert_eq!(
            parse_value(" '3'").unwrap().1,
            " '3'".to_string()
            );

        assert_eq!(
            parse_value(" 'sdt'").unwrap().1,
            " 'sdt'".to_string()
            );
    }

    #[test]
    // get [variables] [connector] [triplets]
    fn test_parse_query_var1() {
        assert_eq!(
            parse_query_var1("get $A where $A est mortel and $A > 4").unwrap().1,
              (vec![Var("A".to_string())], 
               vec![Tvww("A".to_string(), "est".to_string(), "mortel".to_string())], 
               vec![Comp(" $A > 4".to_string())]));
    }

    #[test]
    // get [variables] [connector] [triplets]
    fn test_parse_query_var2() {
        assert_eq!(
            parse_query_var2("get $A where $A est mortel").unwrap().1,
              (vec![Var("A".to_string())], 
               vec![Tvww("A".to_string(), "est".to_string(), "mortel".to_string())], 
               vec![]));
    }

    #[test]
    // get [variables] [connector] [triplets]
    fn test_parse_query_var3() {
        assert_eq!(
            parse_query_var3("get $A where $A > 7").unwrap().1,
              (vec![Var("A".to_string())], 
               vec![Triplet::Empty], 
               vec![Comp(" $A > 7".to_string())]));
    }

    #[test]
    // get [variables] [connector] [triplets]
    fn test_parse_query_var3_2() {
        assert_eq!(
            parse_query_var3("get $A where $A > 7 | ").unwrap(),
              (" | ", 
               (vec![
                Var("A".to_string())],
                vec![Triplet::Empty],
                vec![Comp(" $A > 7".to_string())])));
    }

    #[test]
    fn test_parse_query() {
        if let PredicatAST::Query((var, tri, comp)) = parse_query("get $A where $A > 7").unwrap().1 {
            assert_eq!(var, vec![Var("A".to_string())]);
            assert_eq!(tri, vec![Triplet::Empty]);
            assert_eq!(comp, vec![Comp(" $A > 7".to_string())]);
        }
    }

    #[test]
    fn test_recognize_variable() {
        assert_eq!(
            recognize_variable(" | ").unwrap_or(("", "not")).1,
            "not");
    }

    #[test]
    fn test_parse_value() {
        assert_eq!(
            parse_value(" | ").unwrap_or(("", "not")).1,
            "not");
    }
    
    #[test]
    fn test_parse_valvar() {
        assert_eq!(
            parse_valvar(" | ").unwrap_or(("", "not")).1,
            "not");
    }

}
