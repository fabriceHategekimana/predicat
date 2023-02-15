mod parser;
mod importer;
mod knowledge;
use std::env;

// ajout d'un module parseur

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

fn execute(_sql: &[&String]) -> DataFrame {
    println!("execute function not implemented yet!");
    DataFrame::default()
}

fn develop(_table: DataFrame, res: &[String]) -> Vec<Vec<&String>> {
   res.iter().map(|x| vec![x]).collect::<Vec<Vec<&String>>>()
}

fn parse_and_execute(table: DataFrame, command: &str) -> DataFrame {
    let res = parse_command(command);
    let res = develop(table, &res).into_iter().flatten().collect::<Vec<&String>>();
    println!("res: {:?}", res);
    let _res = execute(&res);
    DataFrame::default()
}

fn main() {
    //TODO add management for multiple commands
    let command = get_args_or("get $A $B such_as $A type $B");
    let df = DataFrame::default();
    let res = parse_and_execute(df, &command);
    println!("res: {:?}", res);
}

