#![allow(dead_code, unused_variables, unused_imports)]

use sqlite::{
        Connection,
        Value,
        Statement,
};

use polars::{
    frame::DataFrame,
    series::Series,
    prelude::NamedFrom
};

use std::collections::HashMap;
use crate::knowledge::Knowledgeable;
use crate::parser;

static SUBJECT: &str = ":subject";
static LINK: &str = ":link";
static GOAL: &str = ":goal";

static CREATE_FACTS : &str = "CREATE TABLE IF NOT EXISTS facts(
                  'subject' TEXT,
                  'link' TEXT,
                  'goal' TEXT,
                  PRIMARY KEY (subject,link,goal)
                ); ";

static CREATE_RULES : &str = "CREATE TABLE IF NOT EXISTS rules_default(
                    'id' INTEGER,
                    'source' TEXT,
                    'type' TEXT, 
                    'listener' TEXT, 
                    'body' TEXT);
                    ";

static CREATE_HISTORICAL : &str = "CREATE TABLE historical(
                            'stage' TEXT, 
                            'event' TEXT,
                            PRIMARY KEY (event)); 
                    ";


static CREATE_MACRO : &str = "CREATE TABLE IF NOT EXISTS macro(
                            'name' TEXT,
                            'body' TEXT);
                    ";

static CREATE_STAGE : &str = "
CREATE TABLE IF NOT EXISTS stage('stage' TEXT); 
";

static CREATE_CONTEXT : &str = "
CREATE TABLE IF NOT EXISTS context('name' TEXT); 
";

static CREATE_UNIQUE_INDEX_RULES : &str = "
CREATE UNIQUE INDEX rules_body on rules (body);
";

static CREATE_UNIQUE_INDEX_FACTS : &str = "CREATE UNIQUE INDEX fact_subject_link_goal ON rules (subject, link, goal);";

static CREATE_UNIQUE_INDEX_HISTORICAL : &str = "
CREATE UNIQUE INDEX historical_event ON historical (event);
";

static INITIALYZE_STAGE : &str = "INSERT INTO stage (stage) VALUES (0)";
static INITIALYZE_CONTEXT : &str = "INSERT INTO context (name) VALUES ('default')";

pub struct SqliteKnowledge {
    connection: Connection
}


impl Knowledgeable for SqliteKnowledge {
    fn new() -> SqliteKnowledge {
        let knowledge = SqliteKnowledge {
            connection: sqlite::open("data.db").unwrap()
        };
        knowledge.modify(&[&CREATE_FACTS.to_string()]);
        knowledge
    }

    fn get(&self, cmds: &[&String]) -> DataFrame {
        for query in cmds {
            let query = query.replace("from facts", "from facts_default");
            let mut hm: HashMap<String, Vec<String>> = HashMap::new();
            let _res = self.connection.iterate(query, |sqlite_couple| {
                for couple in sqlite_couple.iter() {
                    match hm.get_mut(couple.0) {
                        None => hm.insert(couple.0.to_owned(), vec![couple.1.unwrap().to_owned()]),
                        Some(v) => {v.push(couple.1.unwrap().to_owned()); None}
                    };
                }
                true
            });
            let df = to_dataframe(hm);
            println!("df: {:?}", df);
        }
        DataFrame::default()
    }

    fn modify(&self, cmds: &[&String]) {
        let _res = cmds.iter()
            .map(|x| self.connection.execute(x))
            .collect::<Vec<Result<(), sqlite::Error>>>();
    }

    fn translate(&self, s: &parser::PredicatAST) -> &str {
        todo!();
    }

    fn execute<'a>(&self, s: &[&'a str]) -> &'a str {
        todo!();
    }
}


fn add<'l>(connection: &Connection, elements: &[(&str, Value)]) -> Result<(), sqlite::Error> {
    let sql_query = format!(
        "INSERT INTO facts (subject, link, goal) VALUES ({subject}, {link}, {goal});",
        subject=SUBJECT, link=LINK, goal=GOAL);
    let mut statement = connection.prepare(sql_query)?;
    statement.bind::<&[(_, Value)]>(elements)
}

fn to_hashmap<'a>(sqlite_couple: &[(&'a str, &'a str)]) -> HashMap<&'a str, Vec<&'a str>> {
    let mut hm = HashMap::new();
    for couple in sqlite_couple.iter() {
        match hm.get_mut(couple.0) {
            None => hm.insert(couple.0, vec![couple.1]),
            Some(v) => {v.push(couple.1); None}
        };
    }
    hm
}

fn to_dataframe(hm: HashMap<String, Vec<String>>) -> DataFrame {
    let vs = hm.iter().map(|(key, value)| Series::new(key, value)).collect();
    DataFrame::new(vs).unwrap()
}

/*
fn format_triplets(tri: &[Language]) -> String {
    if tri == [Language::Empty]{
        String::from("facts")
    }
    else {
        let sql_queries = tri.iter()
            .filter_map(|x| {
                match x {
                    Tri(t) => Some(triplet_to_sql(&t)),
                    _ => None
                }
            });
        let queries = sql_queries
            .reduce(|acc, x| format!("{} natural join {}", acc, x)).unwrap();
        format!("({})", queries)
    }
}

fn to_sql(res: (&[Language], &[Language], &[Language])) -> String {
    let head = format_variables(&res.0);
    let columns = format_triplets(&res.1); // warning, put the result into a parenthese
    let comparisons = format_comparisons(&res.2);
    format!("{}{}{}", head, columns, comparisons )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn triplet_to_sql(tri: &Triplet) -> String {
        match tri {
            Twww(a,b,c) => 
                format!("SELECT subject,link,goal FROM facts WHERE subject='{}' AND link='{}' AND goal='{}'",a,b,c),
            Tvww(a,b,c) => 
                format!("SELECT subject AS {} FROM facts WHERE link='{}' AND goal='{}'",a,b,c),
            Twvw(a,b,c) => 
                format!("SELECT link AS {} FROM facts WHERE subject='{}' AND goal='{}'",b,a,c),
            Twwv(a,b,c) => 
                format!("SELECT goal AS {} FROM facts WHERE subject='{}' AND link='{}'",c,a,b),
            Tvvw(a,b,c) => 
                format!("SELECT subject AS {},link AS {} FROM facts WHERE goal='{}'",a,b,c),
            Tvwv(a,b,c) => 
                format!("SELECT subject AS {},goal AS {} FROM facts WHERE link='{}'",a,c,b),
            Twvv(a,b,c) => 
                format!("SELECT link AS {},goal AS {} FROM facts WHERE subject='{}'",b,c,a),
            Tvvv(a,b,c) => 
                format!("SELECT subject AS {},link AS {},goal AS {} FROM facts",a,b,c),
        }
    }

    #[test]
    fn test_from_triplet_to_sql() {
        assert_eq!(
            triplet_to_sql(&Tvvv("A","B","C")),
            "SELECT subject AS A,link AS B,goal AS C FROM facts".to_string()
        );
        assert_eq!(
            triplet_to_sql(&Tvwv("A","B","C")),
            "SELECT subject AS A,goal AS C FROM facts WHERE link='B'"
        );
    }
}


    #[test]
    fn test_format_triplets() {
        assert_eq!(
            format_triplets(&vec![Tri(Tvvv("A","B","C"))]),
            "(SELECT subject AS A,link AS B,goal AS C FROM facts)".to_string()
        );
        assert_eq!(
            format_triplets(&vec![Tri(Tvvv("A","B","C")),Tri(Twvv("D","E","F"))]),
            "(SELECT subject AS A,link AS B,goal AS C FROM facts natural join SELECT link AS E,goal AS F FROM facts WHERE subject='D')".to_string()
        );
    }

*/
