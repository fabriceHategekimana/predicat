use parser;
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




fn after_execution(knowledge: &impl Knowledgeable) -> impl Fn(&String) -> () + '_ {
    move |command| {
        if !knowledge.in_cache(command) {
             let cmds = parse(&command);
             execute(&cmds, knowledge);
        }
    }
}

fn parse(command: &str) -> Vec<PredicatAST> {
    parse_command(command).iter()
                .map(PredicatAST::clone)
                .flat_map(substitute_variables(SimpleContext::new()))
                .flatten().collect()
}

fn execute(cmds: &[PredicatAST], knowledge: &impl Knowledgeable) -> Option<SimpleContext> {
    let context = knowledge
            .valid_commands(cmds.to_vec())?
            .iter()
            .map(|x| knowledge.execute_subcommand(x))
            .reduce(SimpleContext::join_contexts)?;

    context.get_aftercmds().iter()
                   .for_each(after_execution(knowledge));

    Some(context.clone())
}

fn main() {
    let input_command = get_args_or("add socrate est mortel");
    let knowledge = new_knowledge("sqlite").expect("Can't open the knowledge!");

    knowledge.clear_cache();

    let cmds = parse(&input_command);
    execute(&cmds, &knowledge)
        .expect("Something went wrong").display();
}
