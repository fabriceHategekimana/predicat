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
use crate::parser::{
                base_parser::Language,
                PredicatAST,
                PredicatAST::{Query, Modifier, Empty}
                };


use crate::parser::base_parser::Language::Word;
use crate::parser::base_parser::Language::Var;
use crate::parser::base_parser::Language::Tri;
use crate::parser::base_parser::Language::Comp;
use crate::parser::base_parser::Triplet::*;
use crate::parser::base_parser::Triplet;

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
        let _ = knowledge.modify(CREATE_FACTS);
        knowledge
    }

    fn get(&self, cmd: &str) -> DataFrame {
        let query = cmd.replace("from facts", "from facts_default");
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
        DataFrame::default()
    }

    fn modify(&self, cmd: &str) -> Result<(), &str>{
        match self.connection.execute(cmd) {
            Ok(r) => Ok(r),
            Err(r) => Err("An error occured with the sqlite database")
        }
    }

    fn translate<'a>(&'a self, asts: &'a [PredicatAST]) -> Vec<Result<(String, &Vec<&str>), &str>> {
        asts.clone().iter().map(|ast| {
            match ast {
                Query((get, link, filter), variables) => Ok(
                    (query_to_sql(get, link, filter),
                    variables)),
                Modifier(commands, variables) => Ok((
                        commands.iter()
                                .fold("".to_string(), |acc, x| format!("{}{}", acc, x)),
                        variables)),
                Empty => Err("The AST is empty") 
            }
        }).collect::<Vec<Result<(String, &Vec<&str>), &str>>>()
    }

    fn execute(&self, s: &Vec<(String, &Vec<&str>)>) -> DataFrame {
        let mut df = DataFrame::default();
        for cmd in s.iter() {
            df = self.execute_helper(df, &cmd.0)
        }
        df
    }

}

impl SqliteKnowledge{
    fn execute_helper(&self, df: DataFrame, s: &str) -> DataFrame {
        println!("s: {:?}", s);
        let mut res = DataFrame::default();
        if &s[0..6]  == "SELECT" {
            res = self.get(s);
        }
        else {
            let _ = self.modify(s);
        }
        res
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

fn query_to_sql(get: &[Language], link: &[Language], filter: &[Language]) -> String {
    let head = format_variables(get);
    let columns = format_triplets(link); // warning, put the result into a parenthese
    let comparisons = format_comparisons(filter);
    format!("{}{}{}", head, columns, comparisons )
}

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


fn format_variables(vars: &[Language]) -> String {
    if vars == [Language::Empty]{
        String::from("SELECT * FROM ")
    }
    else {
        let extracted_vars = vars.iter()
            .filter_map(|x| {
                match x {
                    Var(v) => Some(v),
                    _ => None
                }
            });
        let string_vars = extracted_vars
            .fold("".to_string(), |acc, &x| acc +","+x)
            .chars()
            .skip(1)
            .collect::<String>();
        format!("SELECT {} FROM ",string_vars)
    }
}

fn format_comparisons(comp: &[Language]) -> String {
    if  comp == [Language::Empty] {
        String::from(";")
    }
    else {
        let comparisons = comp.iter()
            .filter_map(|x| {
                match x {
                    Comp(c) => Some(c.replace("$","").replace("==","=")),
                    _ => None
                }
            });
        let final_comparisons = comparisons
            .reduce(|acc, x| format!("{} AND{}", acc, x)).unwrap();
        format!(" WHERE{};", final_comparisons)
    }
}

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

/*
#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_format_variables() {
        assert_eq!(
            format_variables(&vec![Var("X"),Var("Y")]),
            "SELECT X,Y FROM "
        );
        assert_eq!(
            format_variables(&vec![Var("X")]),
            "SELECT X FROM "
        );
    }

    #[test]
    fn test_format_comparisons() {
        assert_eq!(
            format_comparisons(&vec![Comp(" $A == 8")]),
            " WHERE A = 8;".to_string()
        );
        assert_eq!(
            format_comparisons(&vec![Comp(" $A == 8"), Comp(" 6 < 3")]),
            " WHERE A = 8 AND 6 < 3;".to_string()
        );
    }
*/
