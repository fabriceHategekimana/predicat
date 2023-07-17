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

use context::Context;

use parse_modifier::parse_modifier;
pub use self::base_parser::{Language, Triplet};

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

fn substitute_context<'a>(command:&'a str, context: &'a impl Context) -> Vec<String> {
    let variables = extract_variables(command);
    substitute_with_context(command, &variables, context)
}

// TODO apply context
fn apply_context<'a>(variable: &'a str, commands: &'a [String], context: &impl Context) -> Vec<String> {
    let values = &(context.get_values(variable).unwrap());
    let mut res: Vec<String> = vec![];
    for (val, cmd) in values.into_iter().zip(commands){
        let replaced = cmd.replace(&("$".to_string() + variable)[..], val);
        res.push(replaced);
    }
    res
}

fn duplicate_command(command: &str, context: &impl Context) -> Vec<String> {
    (0..=context.len()).into_iter().map(|_x| command.to_string()).collect()
}

fn substitute_with_context<'a>(command: &'a str, variables: &'a [String], context: &impl Context) -> Vec<String> {
    let commands : Vec<String> = duplicate_command(command, context);
    variables.iter()
        .filter(|x| context.is_in_context(x.to_string()))
        .fold(commands, |cmd, x| apply_context(x, &cmd, context))
}

//TODO: test parse command
pub fn parse_command<'a>(string: &'a str, context: &'a impl Context) -> Vec<PredicatAST> {
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
    use super::{
        parse_query,
        Language,
        Triplet,
        apply_context,
    };
    use crate::PredicatAST::Query;
    use polars::prelude::*;
    use polars::df;

    //#[test]
    //fn test_apply_context() {
        //// TODO : apply a mock for the context
        //let df = df![
            //"A" => ["a", "b", "c"]
        //].unwrap();
        //let cmds = vec!["delete $A ami Joe".to_string()];
        //assert_eq!(
            //apply_context("$A", &cmds, &df),
            //vec![
            //"delete a ami Joe",
            //"delete b ami Joe",
            //"delete c ami Joe",
            //]
        //);
    //}

}
