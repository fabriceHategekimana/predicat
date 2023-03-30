use crate::knowledge::sqlite_knowledge::SqliteKnowledge;
use polars::frame::DataFrame;

pub enum Knowledge {
    Sqlite(SqliteKnowledge),
    EmptyKnowledge
}

impl Knowledge {
    fn new(kind: &str) -> Knowledge {
        match kind {
            "sqlite" => Knowledge::Sqlite(SqliteKnowledge::new()),
            _ => Knowledge::EmptyKnowledge
        }
    }

    pub fn get(&self, query: &[&String]) -> DataFrame {
        match self {
            Knowledge::Sqlite(k) => k.get(query),
            Knowledge::EmptyKnowledge =>DataFrame::default()
        }
    }

    pub fn modify(&self, modifier: &[&String]) -> () {
        match self {
            Knowledge::Sqlite(k) => k.modify(modifier),
            Knowledge::EmptyKnowledge => ()
        };
    }

    pub fn translate(&self, s: &str) -> Result<&str, &str> {
        match self {
            Knowledge::Sqlite(k) => Ok(SqliteKnowledge::translate(s)),
            Knowledge::EmptyKnowledge => Err("no knowledge connected") 
        }
    }
}
