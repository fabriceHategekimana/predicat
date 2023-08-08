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

fn get_args_or(query: &str) -> String {
    let args: String = env::args().skip(1)
        .fold(String::new(), |acc, arg| format!("{}{} ", acc, &arg));
    if args == "".to_string() {
        String::from(query)
    }
    else{
        args
    }
}

fn get_context(table: Option<SimpleContext>) -> SimpleContext {
    match table {
        Some(data) => data,
        None => Context::new()
    }
}

fn parse_and_execute(command: &str, knowledge: impl Knowledgeable, table: Option<SimpleContext>) -> SimpleContext {
    let context = get_context(table);
    let ast: Vec<PredicatAST> = parse_command(command, &context); 
    let queries: Vec<String> = knowledge.translate(&ast)
                           .into_iter()
                           .filter_map(|x| x.ok())
                           .collect::<Vec<String>>();
    knowledge.execute(&queries)
}

fn main() {
    let command = get_args_or("add Socrate est mortel");
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let res = parse_and_execute(&command, knowledge, None::<SimpleContext>);
    res.display();
    let mut context = SimpleContext::new();
    context = context.add_column( "name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
    context = context.add_column("age", vec![28.to_string(), 23.to_string(), 28.to_string()]);
    context.display();
    //let context = SimpleContext::from(vec![("A".to_string(), "emy".to_string()), ("B".to_string(), "ami".to_string()), ("C".to_string(), "alice".to_string())]);
    //context.display()
    let mut context = SimpleContext::new();
    context = context.add_column("A", vec!["emy".to_string(),"emy".to_string()]);
    context = context.add_column("B", vec!["ami".to_string(),"ami".to_string()]);
    context = context.add_column("C", vec!["alice".to_string(), "alice".to_string()]);
    context.display();
}
