#![allow(dead_code, unused_variables, unused_imports)]

use sqlite::{
        Connection,
        Value,
        Statement,
};
use polars::{
    df,
    frame::DataFrame,
    series::Series,
    prelude::NamedFrom
};
use std::collections::HashMap;

static SUBJECT: &str = ":subject";
static LINK: &str = ":link";
static GOAL: &str = ":goal";

//TODO: use it when the data test is no more needed
static _CREATE_FACTS : &str = " CREATE TABLE facts(
                  'subject' TEXT,
                  'link' TEXT,
                  'goal' TEXT,
                  PRIMARY KEY (subject,link,goal)
                ); ";

static _CREATE_RULES : &str = " CREATE TABLE rules_default(
                    'id' INTEGER,
                    'source' TEXT,
                    'type' TEXT, 
                    'listener' TEXT, 
                    'body' TEXT);
                    ";

static _CREATE_HISTORICAL : &str = " CREATE TABLE historical(
                            'stage' TEXT, 
                            'event' TEXT,
                            PRIMARY KEY (event)); 
                    ";


static _CREATE_MACRO : &str = " CREATE TABLE macro(
                            'name' TEXT,
                            'body' TEXT);
                    ";

static _CREATE_STAGE : &str = "
CREATE TABLE stage('stage' TEXT); 
";

static _CREATE_CONTEXT : &str = "
CREATE TABLE context('name' TEXT); 
";

static _CREATE_UNIQUE_INDEX_RULES : &str = "
CREATE UNIQUE INDEX rules_body on rules (body);
";

static _CREATE_UNIQUE_INDEX_FACTS : &str = "create unique index fact_subject_link_goal on rules (subject, link, goal);";

static _CREATE_UNIQUE_INDEX_HISTORICAL : &str = "
CREATE UNIQUE INDEX historical_event on historical (event);
";

static _INITIALYZE_STAGE : &str = "insert into stage (stage) values (0)";
static _INITIALYZE_CONTEXT : &str = "insert into context (name) values ('default')";

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
    connection.execute(query).unwrap();
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

pub fn initialisation() {
    sqlite::open("data.db").unwrap();
}
