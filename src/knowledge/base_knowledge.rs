#[derive(PartialEq, Debug)]
pub enum Knowledge {
    SLQ(SqliteKnowledge),
    EmptyKnowledge
}

impl Knowledge {
    fn new(kind: &str) -> Knowledge {
        match kind {
            "sqlite" => SQL(SqliteKnowledge::new()),
            _ => Knowledge::EmptyKnowledge
        }
    }

    fn get(&self, query: &str) -> DataFrame {
        match self {
            SQL(k) => k.get(),
            Knowledge::EmptyKnowledge =>DataFrame::default()
        }
    }

    fn modify(&self, modifier: &str) -> () {
        match self {
            SQL(k) => k.modify(),
            Knowledge::EmptyKnowledge => ()
        };
    }
}
