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


impl SqliteKnowledge {
    fn new() -> SqliteKnowledge {
        SqliteKnowledge {
            connection: sqlite::open("data.db").unwrap()
        }
    }

    pub fn get(&self, cmds: &[&String]) -> DataFrame {
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
}

pub fn get(connection: Connection, query: &str) {
    let query = query.replace("from facts", "from facts_default");
    let mut hm: HashMap<String, Vec<String>> = HashMap::new();
    let _res = connection.iterate(query, |sqlite_couple| {
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

fn modifier(connection: &Connection, query: &str) {
    let res = connection.execute(query);
    match res {
        Ok(s) => (),
        Err(e) => println!("The query '{}' \n failed: \n '{}'", query, e)
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

pub fn initialisation() -> SqliteKnowledge {
    //let connection = sqlite::open("data.db").unwrap();
    let knowledge = SqliteKnowledge::new();
    knowledge.modifier(&[&CREATE_FACTS.to_string()]);
    knowledge
}
