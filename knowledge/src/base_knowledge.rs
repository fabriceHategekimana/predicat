use super::sqlite_knowledge::SqliteKnowledge;
use parser::base_parser::{PredicatAST, Triplet};
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
    fn get_all(&self) -> SimpleContext;
    fn modify(&self, cmds: &str) -> Result<SimpleContext, &str>;
    fn translate<'a>(&'a self, s: &PredicatAST) -> Result<String, &str>;
    fn execute(&self, s: &str) -> SimpleContext;
    fn is_invalid(&self, cmd: &PredicatAST) -> bool;
    fn get_commands_from(&self, cmds: &PredicatAST) -> Vec<String>;
    fn get_command_from_triplet(&self, modifier: &str, tri: &Triplet) -> Vec<String>;
    fn store_modifier(&self, modifier: &PredicatAST);
    fn clear(&self);
} 
