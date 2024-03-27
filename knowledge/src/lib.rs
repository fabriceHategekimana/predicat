mod sqlite_knowledge;
pub mod base_knowledge;

pub use base_knowledge::{
    SqliteKnowledge,
    Knowledgeable,
    new_knowledge,
    Cache,
    RuleManager
};

