#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]
use parser;
use importer;
use knowledge;
use std::env;
use metaprogramming::substitute_variables;

use crate::parser::parse_command;

use crate::knowledge::Knowledgeable;
use crate::knowledge::new_knowledge;
use base_context::Context;
use simple_context::SimpleContext;
use crate::parser::base_parser::PredicatAST;

use serial_test::serial;

fn execute_simple_entry(knowledge: &impl Knowledgeable, cmd: &str) -> () {
    let _: Vec<_> = parse_command(cmd).iter()
        .map(|cmd| {knowledge.store_to_cache(cmd); cmd})
        .flat_map(|cmd| knowledge.translate(&cmd).unwrap_or(vec!["".to_string()]))
        .map(|cmd| knowledge.execute(&cmd))
        .collect();
}

fn get_user_passed_arguments() -> String {
    env::args().skip(1)
        .fold(String::new(), |acc, arg| format!("{}{} ", acc, &arg))
}

fn get_args_or(query: &str) -> String {
    let args = get_user_passed_arguments();
    if args == "".to_string() {
        String::from(query)
    }
    else{
        args
    }
}

fn join_contexts(ctx1: SimpleContext, ctx2: SimpleContext) -> SimpleContext {
    ctx1.join(ctx2)
}

fn has_invalid_commands(cmds: &[PredicatAST], kn: &impl Knowledgeable) -> bool {
    cmds.iter().any(|x| kn.is_invalid(x) == true)
}

fn execute_subcommands(cmds: &[PredicatAST], kn: &impl Knowledgeable) -> SimpleContext {
    cmds.iter()
        .map(|cmd| {kn.store_to_cache(cmd); cmd})
        .flat_map(|cmd| kn.translate(cmd)).flatten()
        .map(|cmd| kn.execute(&cmd))
        .fold(SimpleContext::new(), join_contexts)
}

fn execute_subcommand(kn: &impl Knowledgeable) -> impl FnMut(&PredicatAST) -> SimpleContext + '_ {
    move |subcmd: &PredicatAST| {
    Some(subcmd)
        .map(|cmd| {kn.store_to_cache(&cmd); cmd})
        .map(|cmd| kn.translate(&cmd).unwrap_or(vec!["".to_string()]))
        .unwrap().iter()
        .map(|cmd| kn.execute(&cmd))
        .fold(SimpleContext::new(), join_contexts)}
}

fn concat_sub_commands(cmds: Vec<PredicatAST>, kn: &impl Knowledgeable, aftercmd: Vec<String>) -> Vec<String> {
    cmds.iter().flat_map(|cmd| kn.get_commands_from(cmd))
    .chain(aftercmd.into_iter())
    .collect()
}

fn valid_commands_or_none(kn: &impl Knowledgeable) -> impl Fn(Vec<PredicatAST>) -> Option<Vec<PredicatAST>> + '_ {
    move |cmds: Vec<PredicatAST>|{ 
        cmds.iter().all(|x| !kn.is_invalid(x)).then_some(cmds)
    }
}

type ExecutionState = Option<(SimpleContext, Vec<String>)>;

trait ExecutionStateTrait {
    fn default_value(context: SimpleContext) -> ExecutionState {
        Some((context, vec![]))
    }
}

impl ExecutionStateTrait for ExecutionState { }

fn execute_commands_and_get_after_commands_m(kn: &impl Knowledgeable, aftercmd: Vec<String>) -> impl Fn(Vec<PredicatAST>) -> ExecutionState + '_ {
    move |cmds: Vec<PredicatAST>| {
        Some((execute_subcommands(&cmds, kn),
            concat_sub_commands(cmds, kn, aftercmd.clone())))
    }
}

fn execute_commands_and_get_after_commands_m(kn: &impl Knowledgeable, aftercmd: Vec<String>) -> impl Fn(Vec<PredicatAST>) -> ExecutionState + '_ {
    move |cmds: Vec<PredicatAST>| {
        Some((execute_subcommands(&cmds, kn),
            concat_sub_commands(cmds, kn, aftercmd.clone())))
    }
}

// TODO: bring up the substitution to one level
fn execute_command<'a>(kn: &'a impl Knowledgeable, state: &'a mut ExecutionState) -> impl FnMut(&PredicatAST) -> ExecutionState + 'a {
    move |ast: &PredicatAST| {
        let mut binding = (SimpleContext{ tab: vec![] }, vec![]);
        let (context, aftercmds) = state.as_mut().unwrap_or(&mut binding);
        *state = substitute_variables(ast, &context)
        .map(valid_commands_or_none(kn))?
        .map(execute_commands_and_get_after_commands_m(kn, aftercmds.to_vec()))?;
        state.clone()
    }
}

fn after_execution(knowledge: &impl Knowledgeable) -> impl Fn((SimpleContext, Vec<String>)) -> SimpleContext + '_ {
    move |(context, cmds)| {
        let _ = cmds.iter()
        .filter(|cmd| !knowledge.in_cache(cmd))
        .map(|cmd| parse_and_execute(&cmd, knowledge, SimpleContext::new()))
        .fold(SimpleContext::new(), join_contexts);
        context
    }
}

fn after_execution2(knowledge: &impl Knowledgeable) -> impl Fn(&String) -> () + '_ {
    move |command| {
        if !knowledge.in_cache(command) {
             parse_and_execute(&command, knowledge, SimpleContext::new());
        }
    }
}

fn parse_and_execute(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> SimpleContext {
    let mut state = ExecutionState::default();
    let _ = parse_command(command).iter()
        .map(execute_command(knowledge, &mut state));
        state.map(after_execution(knowledge))
        .unwrap_or_default()
}

fn parse_and_execute2(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> Option<SimpleContext> {
    // TODO : vérifier la partie after cmd
    let state = ExecutionState::default();
    let (mut c, mut acmds) = state.unwrap();
    let cmds = parse_command(command).iter()
        .flat_map(|x| substitute_variables(x, &context))
        .flatten().collect();
    let aftercmds = valid_commands_or_none(knowledge)(cmds)?.iter()
    .map(execute_subcommand(knowledge));
    aftercmds.iter().for_each(after_execution2(knowledge));
    Some(context)
}

fn main() {
    let command = get_args_or("add socrate est mortel");
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let context = SimpleContext::new();
    knowledge.clear_cache();
    let res = parse_and_execute(&command, &knowledge, context);
    res.display();
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::base_parser::Var;
    use parser::Triplet;

    //#[test]
    //#[serial]
    //fn test_execute_simple_entry_add_rule() {
        //let knowledge = new_knowledge("sqlite").unwrap();
        //knowledge.clear();
        //execute_simple_entry(&knowledge, "rule infer add $A ami $B : add $B ami $A");
        //parse_and_execute("add pierre ami emy", &knowledge, SimpleContext::new());
        //let mut context = SimpleContext::new();
        //context = context.add_column("A", &["pierre", "emy"]);
        //context = context.add_column("B", &["ami", "ami"]);
        //context = context.add_column("C", &["emy", "pierre"]);
        //let test_context = knowledge.get_all();
        //let mut sorted_test_context = test_context.get_tab();
        //let mut sorted_context = context.get_tab();
        //sorted_test_context.sort();
        //sorted_context.sort();
        //assert_eq!(
            //sorted_test_context,
            //sorted_context);
    //}

    #[test]
    #[serial]
    fn test_get_command_from_triplet() {
       let knowledge = new_knowledge("sqlite").unwrap();
       knowledge.clear();
       execute_simple_entry(&knowledge, "rule infer add $A ami $B : add $B ami $A");
       let res = knowledge.get_command_from_triplet("add", &Triplet::Tvvv("pierre".to_string(), "ami".to_string(), "emy".to_string()));
        assert_eq!(res,
                   vec!["add emy ami pierre".to_string()]);
    }

    #[test]
    #[serial]
    fn test_cache() {
       let knowledge = new_knowledge("sqlite").unwrap();
       knowledge.clear();
       execute_simple_entry(&knowledge, "add pierre ami emy");
       assert_eq!(
           knowledge.in_cache("add pierre ami emy"),
           true);
    }

    #[test]
    #[serial]
    fn test_delete_command() {
       let knowledge = new_knowledge("sqlite").unwrap();
       knowledge.clear();
       execute_simple_entry(&knowledge, "add pierre ami emy");
       execute_simple_entry(&knowledge, "delete pierre ami emy");
       let res = knowledge.get_all();
        assert_eq!(
            res,
            SimpleContext::new());
    }

    //#[test]
    //#[serial]
    //fn test_associative_rule() {
        //let knowledge = new_knowledge("sqlite").unwrap();
        //knowledge.clear();
        //execute_simple_entry(&knowledge, "rule infer add $A ami $B and $B ami $C : add $A ami $C");
        //execute_simple_entry(&knowledge, "add pierre ami emy");
        //execute_simple_entry(&knowledge, "add emy ami julie");
        //let res = knowledge.get_all();
        //let mut context = SimpleContext::new();
        //context = context.add_column("A", &["pierre", "emy", "pierre"]);
        //context = context.add_column("B", &["ami", "ami", "ami"]);
        //context = context.add_column("C", &["emy", "julie", "julie"]);
        //assert_eq!(res, context);
    //}

}
