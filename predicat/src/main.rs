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
use knowledge::Cache;

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


type ExecutionState = Option<SimpleContext>;

trait ExecutionStateTrait {
    fn default_value(context: SimpleContext) -> ExecutionState {
        Some(context)
    }
}

impl ExecutionStateTrait for ExecutionState { }

fn execute_commands_and_get_after_commands_m(kn: &impl Knowledgeable, aftercmd: Vec<String>) -> impl Fn(Vec<PredicatAST>) -> SimpleContext + '_ {
    move |cmds: Vec<PredicatAST>| {
        SimpleContext{
            tab: execute_subcommands(&cmds, kn).get_tab(),
            cmds: concat_sub_commands(cmds, kn, aftercmd.clone())
        }
    }
}

fn after_execution(knowledge: &impl Knowledgeable) -> impl Fn(&String) -> () + '_ {
    move |command| {
        if !knowledge.in_cache(command) {
             let cmds = parse(&command, knowledge);
             execute(&cmds, knowledge);
        }
    }
}

fn parse(command: &str, knowledge: &impl Knowledgeable) -> Vec<PredicatAST> {
    parse_command(command).iter()
                .map(PredicatAST::clone)
                .flat_map(substitute_variables(SimpleContext::new()))
                .flatten().collect()
}

fn execute(cmds: &[PredicatAST], knowledge: &impl Knowledgeable) -> Option<SimpleContext> {
    let new_contexts = 
        knowledge
        .valid_commands(cmds.to_vec())?
        .iter()
        .map(execute_subcommand(knowledge))
        .collect::<Vec<_>>();

    new_contexts[0].get_aftercmds().iter()
                   .for_each(after_execution(knowledge));

    Some(new_contexts[0].clone())
}

fn main() {
    let input_command = get_args_or("add socrate est mortel");
    let knowledge = new_knowledge("sqlite").expect("Can't open the knowledge!");

    knowledge.clear_cache();

    let cmds = parse(&input_command, &knowledge);
    execute(&cmds, &knowledge)
        .expect("Something went wrong").display();
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::base_parser::Var;
    use parser::Triplet;


    #[test]
    #[serial]
    fn test_get_command_from_triplet() {
       let knowledge = new_knowledge("sqlite").unwrap();
       knowledge.clear_cache_cache();
       execute_simple_entry(&knowledge, "rule infer add $A ami $B : add $B ami $A");
       let res = knowledge.get_command_from_triplet("add", &Triplet::Tvvv("pierre".to_string(), "ami".to_string(), "emy".to_string()));
        assert_eq!(res,
                   vec!["add emy ami pierre".to_string()]);
    }

    #[test]
    #[serial]
    fn test_cache() {
       let knowledge = new_knowledge("sqlite").unwrap();
       knowledge.clear_cache_cache();
       execute_simple_entry(&knowledge, "add pierre ami emy");
       assert_eq!(
           knowledge.in_cache("add pierre ami emy"),
           true);
    }

    #[test]
    #[serial]
    fn test_delete_command() {
       let knowledge = new_knowledge("sqlite").unwrap();
       knowledge.clear_cache_cache();
       execute_simple_entry(&knowledge, "add pierre ami emy");
       execute_simple_entry(&knowledge, "delete pierre ami emy");
       let res = knowledge.get_all();
        assert_eq!(
            res,
            SimpleContext::new());
    }

}
