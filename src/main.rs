mod parser;
mod importer;
mod knowledge;
use std::env;

use polars::frame::DataFrame;
use crate::parser::parse_command;

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

fn develop(_table: DataFrame, res: &[String]) -> Vec<Vec<&String>> {
   res.iter().map(|x| vec![x]).collect::<Vec<Vec<&String>>>()
}

fn parse_and_execute(table: DataFrame, command: &str, Knowledge: Knowledge) -> DataFrame {
    let ast = parse_command(command); // must return an AST
    let queries = knowledge.translate(ast);
    let developped = develop(table, &queries).into_iter().flatten().collect::<Vec<&String>>();
    println!("queries: {:?}", queries);
    let _res = knowledge.execute(&queries); // TODO : vérifier que ça marche
    DataFrame::default()
}

fn main() {
    let command = get_args_or("add Socrate est mortel");
    let df = DataFrame::default();
    let knowledge = knowledge::initialisation();
    let _res = parse_and_execute(df, &command, knowledge);
}

