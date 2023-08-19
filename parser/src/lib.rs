#![allow(dead_code, unused_variables, unused_imports)]

mod parse_modifier;
mod parse_query;
pub mod base_parser;

use regex::Regex;
use itertools::Itertools;
use base_parser::PredicatAST;

use parse_query::{
    parse_query,
    alt
};

use base_context::Context;
use simple_context::SimpleContext;
use nom::multi::many1;
use nom::sequence::terminated;
use nom::bytes::complete::tag;
use nom::IResult;


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

pub fn parse_command<'a>(s: &'a str) -> Vec<PredicatAST> {
    let res = many1(
        alt((
            parse_query_and_modifier,
            parse_query_and_modifier_bar,
            ))
        )(s);
    match res {
        Ok((s, v)) => v,
        Err(e) => vec![]
    }
}

fn is_a_query(s: &str) -> bool {
    s.len() > 0 && &s[0..3] == "get"
}

fn parse_query_and_modifier(s: &str) -> IResult<&str, PredicatAST> {
    let command = s;
    if is_a_query(command) {
       parse_query(command)
    }
    else {
       parse_modifier(command)
    }
}


#[cfg(test)]
mod tests {
    use base_context::Context;
    use simple_context::SimpleContext;
    use crate::base_parser::PredicatAST;

    use super::parse_command;
    use super::extract_variables;
    use super::parse_query_and_modifier;
    use super::parse_query_and_modifier_bar;

    use super::{
        parse_query,
        Language,
        Var,
        Triplet,
        duplicate_command,
    };

    #[test]
    fn test_duplicate_commande() {
        let mut context = SimpleContext::new();
        context = context.add_column("$A", vec!["pierre".to_string(), "anne".to_string(), "murielle".to_string()]);
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

}
