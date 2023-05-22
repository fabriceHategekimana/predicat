mod parser;
mod importer;
mod knowledge;
use std::env;

use polars::frame::DataFrame;
use crate::parser::{
    parse_command,
    PredicatAST,
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

fn get_context(table: Option<DataFrame>) -> DataFrame {
    table.unwrap_or(DataFrame::default())
}

fn parse_and_execute<K>(command: &str, knowledge: K, table: Option<DataFrame>) -> DataFrame 
    where K: Knowledgeable {
    let context = get_context(table);
    let ast: Vec<PredicatAST> = parse_command(command, &context); 
    println!("ast: {:?}", ast);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::base_parser::Language;
    use crate::PredicatAST::Query;
    use crate::parser::Triplet;

    #[test]
    fn main_test(){
        let ast = parse_command("get * such_as $A est mortel", &DataFrame::default());
        assert_eq!(ast,
                   vec![
                   Query((vec![Language::Var("*".to_string())],
                          vec![Language::Tri(Triplet::Twww("Apple".to_string(), "est".to_string(), "mortel".to_string()))],
                          vec![Language::Empty])),
                   Query((vec![Language::Var("*".to_string())],
                          vec![Language::Tri(Triplet::Twww("Apple".to_string(), "est".to_string(), "mortel".to_string()))],
                          vec![Language::Empty])),
                   Query((vec![Language::Var("*".to_string())],
                          vec![Language::Tri(Triplet::Twww("Pear".to_string(), "est".to_string(), "mortel".to_string()))],
                          vec![Language::Empty]))
                   ]);
    }
}
