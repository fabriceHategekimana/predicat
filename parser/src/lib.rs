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

fn substitute_context<'a>(command:&'a str, context: &'a SimpleContext) -> Vec<String> {
    let variables = extract_variables(command);
    substitute_with_context(command, &variables, context)
}

fn apply_context<'a>(variable: &'a str, commands: &'a [String], context: &SimpleContext) -> Vec<String> {
    let values = &(context.get_values(variable).unwrap());
    let mut res: Vec<String> = vec![];
    for (val, cmd) in values.into_iter().zip(commands){
        let replaced = cmd.replace(&(variable)[..], val);
        res.push(replaced);
    }
    res
}

fn duplicate_command(command: &str, context: &SimpleContext) -> Vec<String> {
    (0..context.len()).into_iter().map(|_x| command.to_string()).collect()
}

fn substitute_with_context<'a>(command: &'a str, variables: &'a [String], context: &SimpleContext) -> Vec<String> {
    let commands : Vec<String> = duplicate_command(command, context);
    let new_commands = variables.iter()
        .filter(|x| context.is_in_context(x.to_string()))
        .fold(commands.clone(), |cmd, x| apply_context(x, &cmd, context));
    match new_commands.len() {
        0 => vec![command.to_string()],
        _ => new_commands
    }
}

pub fn parse_command<'a>(string: &'a str, context: &'a SimpleContext) -> Vec<PredicatAST> {
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
    use base_context::Context;
    use simple_context::SimpleContext;
    use crate::base_parser::PredicatAST;

    use super::parse_command;

    use super::{
        parse_query,
        Language,
        Triplet,
        apply_context,
        substitute_with_context,
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
    fn test_substitute_with_context() {
        let mut context = SimpleContext::new();
        context = context.add_column("$A", vec!["pierre".to_string(), "anne".to_string(), "murielle".to_string()]);
        assert_eq!(
            substitute_with_context("add $A ami julie", &["$A".to_string()], &context),
            vec!["add pierre ami julie", "add anne ami julie", "add murielle ami julie"]
                  );
    }

    #[test]
    fn test_apply_context() {
        let mut context = SimpleContext::new();
        context = context.add_column("$A", vec!["pierre".to_string(), "anne".to_string(), "murielle".to_string()]);
        assert_eq!(
            apply_context("$A", &["add $A ami julie".to_string(), "add $A ami julie".to_string(), "add $A ami julie".to_string()], &context),
            vec!["add pierre ami julie", "add anne ami julie", "add murielle ami julie"]
                  );
    }
    
    #[test]
    fn test_parse_command() {
        assert_eq!(
            parse_command("get $A $B $C where $A $B $C", &SimpleContext::new()),
            vec![PredicatAST::Query((
                vec![Language::Var("A".to_string()), Language::Var("B".to_string()), Language::Var("C".to_string())],
                vec![Language::Tri(Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string()))],
                vec![Language::Empty]) 
                  )]);
    }

    #[test]
    fn test_substitute_with_context2() {
        assert_eq!(
            substitute_with_context("add $A ami emi", &["$A".to_string()], &SimpleContext::new()),
            vec!["add $A ami emi"]);
    }

}
