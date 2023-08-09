use parser;
use importer;
use knowledge;
use std::env;

use crate::parser::parse_command;

use crate::knowledge::Knowledgeable;
use crate::knowledge::new_knowledge;
use simple_context::SimpleContext;
use base_context::Context;
use crate::parser::base_parser::PredicatAST;

fn get_args_or(query: &[&str]) -> Vec<String> {
    let args: String = env::args().skip(1)
        .fold(String::new(), |acc, arg| format!("{}{} ", acc, &arg));
    if args == "".to_string() {
        query.iter().map(|x| *x).map(String::from).collect()
    }
    else{
        args.split(" | ").map(String::from).collect()
    }
}

fn parse_and_execute(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> SimpleContext {
    let ast: Vec<PredicatAST> = parse_command(command, &context); 
    let queries: Vec<String> = knowledge.translate(&ast)
                           .into_iter()
                           .filter_map(|x| x.ok())
                           .collect::<Vec<String>>();
    knowledge.execute(&queries, &context)
}

fn main() {
    let commands = get_args_or(&["add Socrate est mortel"]);
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let context = SimpleContext::new();
    let res = commands.iter()
        .fold(context, |context, command| parse_and_execute(command, &knowledge, context));
    res.display();
}
