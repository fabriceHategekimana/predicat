mod parser;
mod importer;
mod knowledge;
use std::env;

// ajout d'un module parseur

use crate::parser::parse_query;

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

fn main() {
    let query = get_args_or("get $A $B such_as $A type $B");
    let res = parse_query(&query);
    println!("res: {:?}", res);
    //let connection = initialisation();
    //let df = DataFrame::new::<Series>(vec![]).unwrap();
    if &res[0..6] == "Select" {
        todo!();
    }
}

