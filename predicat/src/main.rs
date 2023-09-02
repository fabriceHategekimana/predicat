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

fn after_execution(cmds: &[PredicatAST], knowledge: &impl Knowledgeable) -> SimpleContext {
        cmds.iter()
        .flat_map(|x| knowledge.translate(x))
        .map(|cmd| parse_and_execute(&cmd, knowledge, SimpleContext::new()))
        .fold(SimpleContext::new(), join_contexts)
}

fn has_invalid_commands(cmds: &[PredicatAST], kn: &impl Knowledgeable) -> bool {
    cmds.iter().any(|x| kn.is_invalid(x) == true)
}

fn execute_subcommands(cmds: &[PredicatAST], kn: &impl Knowledgeable) -> SimpleContext {
    cmds.iter().flat_map(|cmd| kn.translate(cmd))
                        .map(|cmd| kn.execute(&cmd))
                        .fold(SimpleContext::new(), join_contexts)
}

fn concat_sub_commands(cmds: Vec<PredicatAST>, kn: &impl Knowledgeable, aftercmd: Vec<PredicatAST>) -> Vec<PredicatAST> {
    cmds.iter().flat_map(|cmd| kn.get_commands_from(cmd))
    .flat_map(|cmd| parse_command(&cmd))
    .chain(aftercmd.into_iter())
    .collect()
}

fn execute_command((option_context, aftercmd): (Option<SimpleContext>, Vec<PredicatAST>), ast: &PredicatAST, kn: &impl Knowledgeable) -> (Option<SimpleContext>, Vec<PredicatAST>) {
    let abort = (None, vec![]);

    match option_context {
        None => abort,
        Some(context) => {
            let cmds = substitute(ast, &context);
            match has_invalid_commands(&cmds, kn) {
                true => abort,
                false => (
                        Some(execute_subcommands(&cmds, kn)),
                        concat_sub_commands(cmds, kn, aftercmd))
            }
        }
    }
}

fn parse_and_execute(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> SimpleContext {
    let res = parse_command(command).iter()
        .fold(
            (Some(context), vec![]),
            |entry, cmd| execute_command(entry, cmd, knowledge));

    if let (Some(_), cmds) = res {
        after_execution(&cmds, knowledge)
    } else {
        SimpleContext::new()
    }
}

fn main() {
    let command = get_args_or("add Socrate est mortel");
    //let command = get_args_or("rule before add $A ami $B : add $B ami $A");
    //let command = get_args_or("rule before add $A ami $B : get $A $B where $A ami $B");
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let context = SimpleContext::new();
    let res = parse_and_execute(&command, &knowledge, context);
    res.display();
}
