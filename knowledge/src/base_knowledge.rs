use super::sqlite_knowledge::SqliteKnowledge;
use parser::base_parser::{PredicatAST, Triplet};
use simple_context::SimpleContext;
use base_context::Context;

pub fn new_knowledge(kind: &str) -> Result<impl Knowledgeable, String> {
    match kind {
        "sqlite" => Ok(SqliteKnowledge::new()),
        k => Err(format!("There is no '{}' knowledge availiable", k))
    }
}

pub trait Knowledgeable: Command + FactManager + RuleManager + Cache {
    fn new() -> SqliteKnowledge;

    fn clear_all(&self) {
        self.clear_facts();
        self.clear_rules();
        self.clear_cache();
    }
}

pub trait Cache {
    fn in_cache(&self, cmd: &str) -> bool;
    fn store_to_cache(&self, modifier: &PredicatAST) -> PredicatAST;
    fn clear_cache(&self);
}

pub trait Command: Cache {
    fn get(&self, cmds: &str) -> SimpleContext;
    fn get_all(&self) -> SimpleContext; // get a table of the datas included
    fn modify(&self, cmds: &str) -> Result<SimpleContext, &str>;
    fn translate<'a>(&'a self, s: &PredicatAST) -> Result<Vec<String>, &str>;
    fn execute(&self, s: &str) -> SimpleContext;
    fn is_invalid(&self, cmd: &PredicatAST) -> bool;
    fn get_commands_from(&self, cmds: &PredicatAST) -> Vec<String>;
    fn get_command_from_triplet(&self, modifier: &str, tri: &Triplet) -> Vec<String>;

    fn valid_commands(&self, cmds: Vec<PredicatAST>) -> Option<Vec<PredicatAST>> {
            cmds.iter().all(|x| !self.is_invalid(x)).then_some(cmds)
    }

    fn execute_command(&self, subcmd: &PredicatAST) -> SimpleContext {
        Some(subcmd)
            .map(|cmd| self.store_to_cache(&cmd))
            .map(|cmd| self.translate(&cmd).expect("The translation gone wrong"))
            .unwrap().iter()
            .map(|cmd| self.execute(&cmd))
            .reduce(SimpleContext::join_contexts)
            .expect("The contexts don't have the right contents")
    }
}

pub trait FactManager {
    fn clear_facts(&self);
    fn save_facts(&self, modifier: &str, subject: &str, link: &str, goal: &str);
}

pub trait RuleManager {
    fn store_rule(&self, s: &str) -> SimpleContext; 
    fn get_rules(&self) -> Vec<String>;
    fn clear_rules(&self);
}
