use super::sqlite_knowledge::SqliteKnowledge;
use polars::frame::DataFrame;
use parser::base_parser::PredicatAST;

pub fn new_knowledge(kind: &str) -> Result<impl Knowledgeable, String> {
    match kind {
        "sqlite" => Ok(SqliteKnowledge::new()),
        k => Err(format!("There is no '{}' knowledge availiable", k))
    }
}

pub trait Knowledgeable {
    fn new() -> SqliteKnowledge;
    fn get(&self, cmds: &str) -> DataFrame;
    fn modify(&self, cmds: &str) -> Result<(), &str>;
    fn translate<'a>(&'a self, s: &'a [PredicatAST]) -> Vec<Result<String, &str>>;
    fn execute(&self, s: &Vec<String>) -> DataFrame;
} 
