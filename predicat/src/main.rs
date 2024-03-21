use parser;
use std::env;
use knowledge;
use knowledge::Cache;
use base_context::Context;
use simple_context::SimpleContext;
use crate::knowledge::Knowledgeable;
use crate::knowledge::new_knowledge;
use metaprogramming::substitute_variables;
use crate::parser::base_parser::PredicatAST;
use crate::parser::parse_command;


fn parse(command: &String) -> Vec<PredicatAST> {
    parse_command(command).iter()
                .map(PredicatAST::clone)
                .flat_map(substitute_variables(SimpleContext::new()))
                .flatten().collect()
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

fn execute(cmds: &[PredicatAST], knowledge: &impl Knowledgeable) -> Option<SimpleContext> {
    let context = knowledge
            .valid_commands(cmds.to_vec())?
            .iter()
            .map(|x| knowledge.execute_command(x))
            .reduce(SimpleContext::join_contexts)?;

    Some(context.clone())
}

fn interpret(cmds: &[String], knowledge: &impl Knowledgeable) -> SimpleContext {
    let cmds = cmds.iter()
                .flat_map(parse)
                .collect::<Vec<PredicatAST>>();
    execute(&cmds, knowledge)
                .expect("Something went wrong")
}

fn main() {
    let input_command = get_args_or("add socrate est mortel");
    let knowledge = new_knowledge("sqlite").expect("Can't open the knowledge!");

    knowledge.clear_cache();

    let mut context = interpret(&vec![input_command], &knowledge);

    while context.has_commands() && !context.has_error() {
        context = interpret(&context.get_aftercmds(), &knowledge);
    }

    context.display(); //display context or error
}
