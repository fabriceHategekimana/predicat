#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]
use parser;
use importer;
use knowledge;
use std::env;
use metaprogramming::substitute;

use crate::parser::parse_command;

use crate::knowledge::Knowledgeable;
use crate::knowledge::new_knowledge;
use base_context::Context;
use simple_context::SimpleContext;
use crate::parser::base_parser::PredicatAST;

use serial_test::serial;

fn execute_simple_entry(knowledge: &impl Knowledgeable, cmd: &str) -> () {
    let _: Vec<_> = parse_command(cmd).iter()
        .map(|x| knowledge.translate(&x).unwrap_or("".to_string()))
        .map(|x| knowledge.execute(&x))
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

fn after_execution(cmds: &[String], knowledge: &impl Knowledgeable) {
        let _ = cmds.iter()
        .map(|cmd| parse_and_execute(&cmd, knowledge, SimpleContext::new()))
        .fold(SimpleContext::new(), join_contexts);
}

fn has_invalid_commands(cmds: &[PredicatAST], kn: &impl Knowledgeable) -> bool {
    cmds.iter().any(|x| kn.is_invalid(x) == true)
}

fn execute_subcommands(cmds: &[PredicatAST], kn: &impl Knowledgeable) -> SimpleContext {
    cmds.iter().flat_map(|cmd| kn.translate(cmd))
                        .map(|cmd| kn.execute(&cmd))
                        .fold(SimpleContext::new(), join_contexts)
}

fn concat_sub_commands(cmds: Vec<PredicatAST>, kn: &impl Knowledgeable, aftercmd: Vec<String>) -> Vec<String> {
    cmds.iter().flat_map(|cmd| kn.get_commands_from(cmd))
    //.flat_map(|cmd| parse_command(&cmd))
    .chain(aftercmd.into_iter())
    .collect()
}

fn valid_commands_or_none(cmds: Vec<PredicatAST>, kn: &impl Knowledgeable) -> Option<Vec<PredicatAST>> {
    cmds.iter().all(|x| !kn.is_invalid(x)).then_some(cmds)
}

type ExecutionState = Option<(SimpleContext, Vec<String>)>;

fn execute_command_helper(cmds: Vec<PredicatAST>, kn: &impl Knowledgeable, aftercmd: Vec<String>) -> ExecutionState {
    Some((execute_subcommands(&cmds, kn),
        concat_sub_commands(cmds, kn, aftercmd)))
}

fn execute_command(context_aftercmds: ExecutionState, ast: &PredicatAST, kn: &impl Knowledgeable) -> ExecutionState {
    context_aftercmds.map(|(context, aftercmds)| {
        substitute(ast, &context).map(|cmds| 
        valid_commands_or_none(cmds, kn)).unwrap_or(None).map(|cmds| 
        execute_command_helper(cmds, kn, aftercmds)).unwrap_or(None)}).unwrap_or(None)
}


fn parse_and_execute(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> SimpleContext {
    parse_command(command).iter().fold(Some((context, vec![])), |entry, cmd|
        execute_command(entry, cmd, knowledge))
        .map(|(context, aftercmds)| { after_execution(&aftercmds, knowledge); context })
        .unwrap_or_default()
}

fn main() {
    let command = get_args_or("add socrate est mortel");
    //let command = get_args_or("get $A $B $C where $A $B $C");
    //let command = get_args_or("rule block add $A ami $B : add $B ami $A");
    //let command = get_args_or("rule infer add $A ami $B : get $A $B where $A ami $B");
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let context = SimpleContext::new();
    let res = parse_and_execute(&command, &knowledge, context);
    res.display();
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::base_parser::Var;
    use parser::Triplet;

    #[test]
    #[serial]
    fn test_get() {
        let knowledge = new_knowledge("sqlite").unwrap();
        knowledge.clear();
        execute_simple_entry(&knowledge, "add socrate est mortel");
        let mut context = SimpleContext::new();
        context = context.add_column("A", &["socrate"]);
        context = context.add_column("B", &["est"]);
        context = context.add_column("C", &["mortel"]);
        let test_context = knowledge.get_all();
        assert_eq!(test_context, context);
    }

    #[test]
    #[serial]
    fn test_execute_simple_entry_add_rule() {
        let knowledge = new_knowledge("sqlite").unwrap();
        knowledge.clear();
        execute_simple_entry(&knowledge, "rule infer add $A ami $B : add $B ami $A");
        parse_and_execute("add pierre ami emy", &knowledge, SimpleContext::new());
        let mut context = SimpleContext::new();
        context = context.add_column("A", &["pierre", "emy"]);
        context = context.add_column("B", &["ami", "ami"]);
        context = context.add_column("C", &["emy", "pierre"]);
        let test_context = knowledge.get_all();
        assert_eq!(test_context, context);
    }

    #[test]
    #[serial]
    fn test_get_command_from_triplet() {
       let knowledge = new_knowledge("sqlite").unwrap();
       knowledge.clear();
       execute_simple_entry(&knowledge, "rule infer add $A ami $B : add $B ami $A");
       let res = knowledge.get_command_from_triplet("add", &Triplet::Tvvv("pierre".to_string(), "ami".to_string(), "emy".to_string()));
        assert_eq!(res,
                   vec!["add $B ami $A".to_string()]);
    }

}
