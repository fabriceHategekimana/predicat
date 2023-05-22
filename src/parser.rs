#![allow(dead_code, unused_variables, unused_imports)]

mod parse_modifier;
mod parse_query;
pub mod base_parser;

use polars::frame::DataFrame;
use polars::df;
use polars::prelude::NamedFrom;
use crate::PredicatAST::Query;

use regex::Regex;
use itertools::Itertools;

use parse_query::{
    parse_query,
    alt
};

use parse_modifier::parse_modifier;

pub use self::base_parser::{Language, Triplet};

#[derive(PartialEq, Debug)]
pub enum PredicatAST {
    Query(
        (Vec<Language>,
         Vec<Language>,
         Vec<Language>)),
    Modifier(Vec<String>),
    Empty,
    Debug(String)
}

fn soft_predicat(s: &str) -> &str {
    s
}


fn extract_variables(command: &str) -> Vec<String> {
    let re = Regex::new(r"\$(?P<variable>\w)").unwrap();
    re.captures_iter(command)
        .map(|x| x["variable"].to_owned())
        .unique()
        .collect()
}

fn substitute_context<'a>(command:&'a str, _context: &'a DataFrame) -> Vec<String> {
    let variables = extract_variables(command);
    let context = df!("A" => &["Apple", "Apple", "Pear"],
                 "B" => &["Red", "Yellow", "Green"]).unwrap();
    substitute_with_context(command, &variables, &context)
}


fn apply_context<'a>(variable: &'a str, commands: &'a [String], context: &DataFrame) -> Vec<String> {
    let values = &(context.select_series(&[variable]).unwrap()[0]);
    let mut res: Vec<String> = vec![];
    for (val, cmd) in values.utf8().unwrap().into_iter().zip(commands){
        let replaced = cmd.replace(&("$".to_string() + variable)[..], val.unwrap());
        res.push(replaced);
    }
    res
}

fn exist_in(variable: &str, context: &DataFrame) -> bool {
    context.get_column_names().iter().any(|x| x == &variable)
}

fn duplicate_command(command: &str, context: &DataFrame) -> Vec<String> {
    (0..=context.shape().1).into_iter().map(|_x| command.to_string()).collect()
}

fn substitute_with_context<'a>(command: &'a str, variables: &'a [String], context: &DataFrame) -> Vec<String> {
    let commands : Vec<String> = duplicate_command(command, context);
    variables.iter()
        .filter(|x| exist_in(&x, &context))
        .fold(commands, |cmd, x| apply_context(x, &cmd, &context))
}

//main
pub fn parse_command<'a>(string: &'a str, context: &'a DataFrame) -> Vec<PredicatAST> {
    string.split(" | ")
          .map(soft_predicat)
          .flat_map(|x| substitute_context(x, context))
          .map(parse_query_and_modifier)
          .collect::<Vec<PredicatAST>>()
}

fn is_a_query(s: &str) -> bool {
    &s[0..3] == "get"
}

fn parse_query_and_modifier(s: String) -> PredicatAST {
    let command = &(s.clone())[..];
    if is_a_query(command) {
       parse_query(command)
    }
    else {
       parse_modifier(command)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        assert_eq!(parse_query("get $A such_as $A ami Bob $A == 7"),
                   Query((vec![Language::Var("A".to_string())], vec![Language::Tri(Triplet::Tvww("A".to_string(),"ami".to_string(),"Bob".to_string()))], vec![Language::Comp(" $A == 7".to_string())]))
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7"),
                   Query((vec![Language::Var("A".to_string()), Language::Var("B".to_string())], vec![Language::Tri(Triplet::Tvwv("A".to_string(),"ami".to_string(),"B".to_string()))], vec![Language::Comp(" $A == 7".to_string())]))
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7 and $B < 9"),
                   Query((vec![Language::Var("A".to_string()), Language::Var("B".to_string())], vec![Language::Tri(Triplet::Tvwv("A".to_string(),"ami".to_string(),"B".to_string()))], vec![Language::Comp(" $A == 7".to_string()), Language::Comp(" $B < 9".to_string())]))
                   );
    }
}
