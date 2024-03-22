#![allow(dead_code, unused_variables, unused_imports)]

mod parse_modifier;
mod parse_query;
pub mod base_parser;

use regex::Regex;
use itertools::Itertools;
use base_parser::PredicatAST;
use base_parser::Action;
use base_parser::CommandType;
use base_parser::Command;

use parse_query::{
    parse_query,
    alt
};

use base_context::Context;
use simple_context::SimpleContext;
use nom::multi::many1;
use nom::sequence::terminated;
use nom::bytes::complete::tag;
use nom::sequence::tuple;
use nom::IResult;
use nom::combinator::recognize;


use parse_modifier::parse_modifier;
pub use self::base_parser::{Language, Var, Triplet, parse_bar};

use crate::Triplet::*;

pub fn soft_predicat(s: &str) -> &str {
    s
}


fn extract_variables(command: &str) -> Vec<String> {
    let re = Regex::new(r"\$(?P<variable>\w)").unwrap();
    re.captures_iter(command)
        .map(|x| x["variable"].to_owned())
        .unique()
        .collect()
}

fn duplicate_command(command: &str, context: &SimpleContext) -> Vec<String> {
    (0..context.len()).into_iter().map(|_x| command.to_string()).collect()
}

fn parse_query_and_modifier_bar(s: &str) -> IResult<&str, PredicatAST> {
    terminated(parse_query_and_modifier, parse_bar)(s)
}

fn parse_action(s: &str) -> IResult<&str, Action> {
    let res = alt((tag("block "), (tag("infer "))))(s);
    match res {
        Ok((s, "block ")) => Ok((s, Action::Block)),
        Ok((s, "infer ")) => Ok((s, Action::Infer)),
        Err(r) => Err(r),
        _ => todo!()
    }
}

fn parse_trigger(s: &str) -> IResult<&str, (CommandType, Vec<Triplet>)> {
    let res = parse_modifier(s);
    match res {
        Ok((s, PredicatAST::AddModifier(v))) => Ok((s, (CommandType::Add, v.clone()))),
        Ok((s, PredicatAST::DeleteModifier(v))) => Ok((s, (CommandType::Delete, v.clone()))),
        Err(r) => Err(r),
        _ => todo!()
    }
}

fn parse_cmd(s: &str) -> IResult<&str, (String, Box<PredicatAST>)> {
    let res = recognize(parse_query_and_modifier)(s);
    match res {
        Ok((s, st)) => Ok((s, (
                    st.to_string(),
                    Box::new(parse_query_and_modifier(st).unwrap().1)))),
        Err(r) => Err(r)
    }
}

fn parse_rule(s: &str) -> IResult<&str, PredicatAST> {
    let res = tuple((
            tag("infer "),
            parse_trigger,
            tag(" -> "),
            parse_cmd
          ))(s);
    match res {
        Ok((s, (r, (ty, tri), _, (st, ast)))) => Ok((s, PredicatAST::Infer((ty, tri), (st, ast)))),
        Err(r) => Err(r)
    }
}

pub fn parse_command<'a>(s: &'a str) -> Vec<PredicatAST> {
    let res = many1(
        alt((
            parse_query_and_modifier,
            parse_query_and_modifier_bar,
            parse_rule
            ))
        )(s);
    match res {
        Ok((s, v)) => { 
            //println!("{:?}", &v);
            v},
        Err(e) => { println!("{:?}", e); vec![] }
    }
}

fn is_a_query(s: &str) -> bool {
    s.len() > 3 && &s[0..3] == "get"
}

fn parse_query_and_modifier(s: &str) -> IResult<&str, PredicatAST> {
    if is_a_query(s) {
       parse_query(s)
    }
    else {
       parse_modifier(s)
    }
}


#[cfg(test)]
mod tests {
    use base_context::Context;
    use simple_context::SimpleContext;
    use crate::base_parser::PredicatAST;

    use super::*;

    #[test]
    fn test_duplicate_commande() {
        let mut context = SimpleContext::new();
        context = context.add_column("$A", &["pierre", "anne", "murielle"]);
        assert_eq!(
            context.len(),
            3);
        assert_eq!(
            duplicate_command("add $A ami julie", &context),
            vec!["add $A ami julie".to_string(), "add $A ami julie".to_string(), "add $A ami julie".to_string()]
                  );
    }


    #[test]
    fn test_parse_query_and_modifier() {
        assert_eq!(
            parse_query_and_modifier("get $A $B $C where $A $B $C").unwrap().1,
            PredicatAST::Query((
                vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                vec![]) 
                  ));
    }

    #[test]
    fn test_parse_query_and_modifier2() {
        assert_eq!(
            parse_query_and_modifier("get $A $B $C where $A $B $C").unwrap().1,
            PredicatAST::Query((
                vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                vec![]) 
                  ));
    }

    #[test]
    fn test_parse_query_and_modifier_bar() {
        assert_eq!(
            parse_query_and_modifier_bar("get $A $B $C where $A $B $C | ").unwrap().1,
            PredicatAST::Query((
                vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                vec![]) 
                  ));
    }
    
    #[test]
    fn test_parse_command() {
        assert_eq!(
            parse_command("get $A $B $C where $A $B $C"),
            vec![PredicatAST::Query((
                vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                vec![]) 
                  )]);
    }

    #[test]
    fn test_extract_variable() {
        assert_eq!(
            extract_variables("add $C ami julie"),
            vec!["C".to_string()]);
    }

    #[test]
    fn test_parse_rule() {
        assert_eq!(
            parse_rule("rule block add $A ami $B : get $A $B where $A ami $B").unwrap().1,
            PredicatAST::Infer(
                              (CommandType::Add, vec![Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string())]), ("get $A $B where $A ami $B".to_string(), Box::new(PredicatAST::Query((vec![Var("A".to_string()), Var("B".to_string())], vec![Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string())], vec![]))))),
                  );
    }

}
