#![allow(dead_code, unused_variables, unused_imports)]

mod parse_modifier;
mod parse_query;
pub mod base_parser;

use regex::Regex;
use itertools::Itertools;
use base_parser::PredicatAST;
use base_parser::Event;
use base_parser::Action;
use base_parser::ModifierType;

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

fn parse_event(s: &str) -> IResult<&str, Event> {
    let res = alt((tag("before "), (tag("after "))))(s);
    match res {
        Ok((s, "before ")) => Ok((s, Event::Before)),
        Ok((s, "after ")) => Ok((s, Event::After)),
        Err(r) => Err(r),
        _ => todo!()
    }
}

fn parse_trigger(s: &str) -> IResult<&str, (ModifierType, Triplet)> {
    let res = parse_modifier(s);
    match res {
        Ok((s, PredicatAST::AddModifier(v))) => Ok((s, (ModifierType::Add, v[0].clone()))),
        Ok((s, PredicatAST::DeleteModifier(v))) => Ok((s, (ModifierType::Delete, v[0].clone()))),
        Err(r) => Err(r),
        _ => todo!()
    }
}

fn parse_action(s: &str) -> IResult<&str, Action> {
    let res = alt((
    recognize(parse_query_and_modifier),
    tag("block"),
        ))(s);
    match res {
        Ok((s, "block")) => Ok((s, Action::Block)),
        Ok((s, cmd)) => Ok((s, Action::Command(cmd.to_string()))),
        Err(r) => Err(r)
    }
}

fn parse_rule(s: &str) -> IResult<&str, PredicatAST> {
    // rule [event] [trigger] [action] 
    let res = tuple((
            tag("rule "),
            parse_event,
            parse_trigger,
            tag(" : "),
            parse_action
          ))(s);
    match res {
        Ok((s, (r, e, (ty, tri), _, a))) => Ok((s, PredicatAST::Rule(e, (ty, tri), a))),
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
        Ok((s, v)) => v,
        Err(e) => vec![]
    }
}

fn is_a_query(s: &str) -> bool {
    s.len() > 0 && &s[0..3] == "get"
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

    //use super::parse_command;
    //use super::extract_variables;
    //use super::parse_query_and_modifier;
    //use super::parse_query_and_modifier_bar;

    //use super::{
        //parse_query,
        //Language,
        //Var,
        //Triplet,
        //duplicate_command,
    //};

    use super::*;

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

    #[test]
    fn test_parse_action() {
        let a = "add $B ami $A";
        assert_eq!(
            parse_action(a).unwrap().1,
            Action::Command("add $B ami $A".to_string())
                  );
    }

    #[test]
    fn test_parse_rule1() {
        assert_eq!(
            parse_rule("rule before add $A ami $B : add $B ami $A").unwrap().1,
            PredicatAST::Rule(
                Event::Before,
                (ModifierType::Add, Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string())),
                Action::Command("add $B ami $A".to_string()))       
                  );
    }

}
