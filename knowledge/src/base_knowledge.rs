pub use super::sqlite_knowledge::SqliteKnowledge;
use parser::base_parser::{PredicatAST, Triplet};
use base_context::simple_context::DataFrame;


pub fn new_knowledge<K: Knowledgeable<DataFrame>>(kind: &str) -> Result<K, String> {
    match kind {
        "sqlite" => Ok(K::new()),
        k => Err(format!("There is no '{}' knowledge availiable", k))
    }
}

pub trait Knowledgeable<T: Joinable + Clone>: Command<T> + FactManager + RuleManager<T> + Cache {
    fn new() -> Self;

    fn clear_all(&self) {
        self.clear_facts();
        self.clear_rules();
        self.clear_cache();
    }
}

pub trait Cache {
    fn in_cache(&self, cmd: &PredicatAST) -> bool;
    fn store_to_cache(&self, modifier: &PredicatAST) -> PredicatAST;
    fn clear_cache(&self);
}

pub trait Joinable {
    fn join(a: Self, b: Self) -> Self;
}

pub trait Command<Data: Joinable + Clone>: Cache {
    type Language;
    fn get(&self, cmds: &str) -> Data;
    fn get_all(&self) -> Data; // get a table of the datas included
    fn modify(&self, cmds: &str) -> Result<Data, &str>;
    fn translate<'a>(&'a self, s: &PredicatAST) -> Result<Vec<Self::Language>, &str>;
    fn execute(&self, s: &Self::Language) -> Data;
    fn is_invalid(&self, cmd: &PredicatAST) -> bool;
    fn infer_commands_from(&self, cmds: &PredicatAST) -> Vec<String>;
    fn infer_command_from_triplet(&self, modifier: &str, tri: &Triplet) -> Vec<String>;

    fn valid_commands(&self, cmds: Vec<PredicatAST>) -> Option<Vec<PredicatAST>> {
            cmds.iter().all(|x| !self.is_invalid(x)).then_some(cmds)
    }

    fn execute_command(&self, subcmd: &PredicatAST) -> Data {
        Some(subcmd)
            .map(|cmd| self.store_to_cache(&cmd))
            .map(|cmd| self.translate(&cmd).expect("The translation gone wrong"))
            .unwrap().iter()
            .map(|cmd| self.execute(cmd))
            .reduce(Data::join)
            .expect("The contexts don't have the right contents")
    }
}

pub trait FactManager {
    fn clear_facts(&self);
    fn save_facts(&self, modifier: &str, subject: &str, link: &str, goal: &str);
}

pub trait RuleManager<Data> {
    fn store_rule(&self, s: &str) -> Data; 
    fn get_rules(&self) -> Vec<String>;
    fn clear_rules(&self);
}
