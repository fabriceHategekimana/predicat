use super::sqlite_knowledge::SqliteKnowledge;
use parser::base_parser::PredicatAST;
use simple_context::SimpleContext;

pub fn new_knowledge(kind: &str) -> Result<impl Knowledgeable, String> {
    match kind {
        "sqlite" => Ok(SqliteKnowledge::new()),
        k => Err(format!("There is no '{}' knowledge availiable", k))
    }
}

pub trait Knowledgeable {
    fn new() -> SqliteKnowledge;
    fn get(&self, cmds: &str) -> SimpleContext;
    fn modify(&self, cmds: &str) -> Result<SimpleContext, &str>;
    fn translate<'a>(&'a self, s: &PredicatAST) -> Result<String, &str>;
    fn execute(&self, s: &str) -> SimpleContext;
} 
