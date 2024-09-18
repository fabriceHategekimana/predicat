#![allow(dead_code, unused_variables, unused_imports)]

pub mod parse_modifier;
mod parse_query;
mod triplet;

pub mod var;
pub mod cmd;
pub mod base_parser;

pub use cmd::Cmd;
pub use var::Var;
pub use triplet::Triplet;
pub use self::base_parser::{Language, parse_bar};

use nom::Err::Error;
use regex::Regex;
use base_parser::Action;
use itertools::Itertools;
use base_parser::PredicatAST;
use base_parser::CommandType;
use base_parser::Command;
use parse_query::{
    parse_query,
    alt
};

use nom::combinator::peek;
use nom::multi::many1;
use nom::sequence::terminated;
use nom::bytes::complete::tag;
use nom::sequence::tuple;
use nom::IResult;
use nom::combinator::recognize;
use nom::combinator::consumed;
use parse_modifier::parse_modifier;

pub fn soft_predicat(s: &str) -> &str {
    s
}

fn parse_query_and_modifier_bar(s: &str) -> IResult<&str, PredicatAST> {
    terminated(parse_query_and_modifier, parse_bar)(s)
}

//fn parse_trigger(s: &str) -> IResult<&str, (CommandType, Vec<Triplet>, &str)> {
fn parse_trigger(s: &str) -> IResult<&str, &str> {
    let res = consumed(parse_modifier)(s);
    match res {
        Ok((so, (s1, PredicatAST::Modifier(CommandType::Add, v)))) => Ok((so, s1)),
        Ok((so, (s1, PredicatAST::Modifier(CommandType::Delete, v)))) => Ok((so, s1)),
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

fn parse_infer(s: &str) -> IResult<&str, PredicatAST> {
    let res = tuple((
            tag("infer "),
            parse_trigger,
            tag(" -> "),
            recognize(parse_cmd)
          ))(s);
    match res {
        Ok((s, (_, pre_conditions, _, post_conditions))) 
            => Ok((s, PredicatAST::Infer(pre_conditions.to_string(), post_conditions.to_string()))),
        Err(r) => Err(r),
    }
}

pub fn parse_command(c: &Cmd) -> Vec<PredicatAST> {
    let res = many1(
        alt((
            parse_query_and_modifier_bar,
            parse_query_and_modifier,
            parse_infer
            // TODO: add validation rule
            ))
        )(&*c);
    match res {
        Ok((s, v)) => { 
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
    use crate::base_parser::PredicatAST;

    use super::*;

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
            parse_command(&Cmd::new("get $A $B $C where $A $B $C")),
            vec![PredicatAST::Query((
                vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                vec![]) 
                  )]);
    }

}
