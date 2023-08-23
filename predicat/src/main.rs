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


fn parse_and_execute(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> SimpleContext {
    let asts: Vec<PredicatAST> = parse_command(command); 
    let translate = |x| knowledge.translate(&x);
    let execute = |x: String| knowledge.execute(&x);
    asts.iter().fold(context, |ctx, ast| {
       substitute(ast, ctx).into_iter()
               .flat_map(translate)
               .map(execute)
               .fold(SimpleContext::new(), |ctx1, ctx2| ctx1.join(ctx2))
    })
}

fn main() {
    let command = get_args_or("add Socrate est mortel");
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let context = SimpleContext::new();
    let res = parse_and_execute(&command, &knowledge, context);
    res.display();
}
