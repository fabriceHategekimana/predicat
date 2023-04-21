use crate::knowledge::sqlite_knowledge::SqliteKnowledge;
use polars::frame::DataFrame;
use crate::parser;

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
    fn translate(&self, s: &[parser::PredicatAST]) -> Vec<Result<(String, &Vec<&str>), &str>>;
    fn execute(&self, s: &[(String, &str)]) -> DataFrame;
} 
