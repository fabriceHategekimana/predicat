mod parser;
mod importer;
mod knowledge;
use std::env;

use polars::frame::DataFrame;
use crate::parser::{
    parse_command,
    PredicatAST
};

use crate::knowledge::Knowledgeable;
use crate::knowledge::new_knowledge;

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

fn parse_and_execute<K>(command: &str, knowledge: K, table: Option<DataFrame>) -> DataFrame 
    where K: Knowledgeable {
    let context = table.unwrap_or(DataFrame::default());
    let ast: Vec<PredicatAST> = parse_command(command, context); 
    let queries = knowledge.translate(&ast)
                           .into_iter()
                           .filter_map(|x| x.ok())
                           .collect::<Vec<String>>();
    knowledge.execute(&queries)
}

fn main() {
    let command = get_args_or("add Socrate est mortel");
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let res = parse_and_execute(&command, knowledge, None);
    println!("res: {:?}", res);
}
