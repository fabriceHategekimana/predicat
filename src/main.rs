mod parser;
mod importer;
mod knowledge;
use std::env;

use polars::frame::DataFrame;
use crate::parser::parse_command;
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

fn parse_and_execute<K: Knowledgeable>(_table: DataFrame, command: &str, knowledge: K) -> DataFrame {
    let ast: parser::PredicatAST = parse_command(command); 
    let queries = knowledge.translate(&ast);
    let _res = knowledge.execute(&[queries]); // TODO : vérifier que ça marche
    DataFrame::default()
}

fn main() {
    let command = get_args_or("add Socrate est mortel");
    let df = DataFrame::default();
    let knowledge = new_knowledge("sqlite");
    let _res = match knowledge {
        Ok(k) => parse_and_execute(df, &command, k),
        _ => DataFrame::default()
    };
}
