#![allow(dead_code, unused_variables, unused_imports)]

use sqlite::{
        Connection,
        Value,
        Statement,
};

use simple_context::SimpleContext;
use base_context::Context;

use std::collections::HashMap;
use super::Knowledgeable;

use parser::soft_predicat;

use parser::base_parser::PredicatAST;
use parser::base_parser::PredicatAST::{Query, AddModifier, DeleteModifier, Empty};

use parser::base_parser::Var;
use parser::base_parser::Language;
use parser::base_parser::Language::Word;
use parser::base_parser::Language::Tri;
use parser::base_parser::Comp;
use parser::base_parser::Triplet::*;
use parser::base_parser::Triplet;

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

static INITIALYZE_STAGE : &str = "INSERT or IGNORE INTO stage (stage) VALUES (0)";
static INITIALYZE_CONTEXT : &str = "INSERT or IGNORE INTO context (name) VALUES ('default')";

pub struct SqliteKnowledge {
    connection: Connection
}

fn extract_columns(sql_select_query: &str) -> Vec<&str> {
   sql_select_query.split_once(" FROM")
                   .unwrap().0
                   .split_once(" ")
                   .unwrap().1
                   .split(",")
                   .collect()
}

impl Knowledgeable for SqliteKnowledge {
    fn new() -> SqliteKnowledge {
        let knowledge = SqliteKnowledge {
            connection: sqlite::open("data.db").unwrap()
        };
        let _ = knowledge.modify(CREATE_FACTS);
        knowledge
    }

    fn get(&self, cmd: &str) -> SimpleContext {
        let query = cmd;
        let mut hm: HashMap<String, Vec<String>> = HashMap::new();
        let _ = self.connection.iterate(query, |sqlite_couple| {
            for couple in sqlite_couple.iter() {
                match hm.get_mut(couple.0) {
                    None => hm.insert(couple.0.to_owned(), vec![couple.1.unwrap().to_owned()]),
                    Some(v) => {v.push(couple.1.unwrap().to_owned()); None}
                };
            }
            true
        });
        let sc = to_context(hm, extract_columns(query));
        sc
    }

    fn modify(&self, cmd: &str) -> Result<SimpleContext, &str> {
        match self.connection.execute(cmd) {
            Ok(r) => Ok(SimpleContext::new()),
            Err(r) => {println!("r: {:?}", r); Err("An error occured with the sqlite database")}
        }
    }


    fn translate<'a>(&'a self, ast: &PredicatAST) -> Result<String, &str> {
        match ast {
            Query((get, link, filter)) => Ok(query_to_sql(get, link, filter)),
            AddModifier(commands) => 
                Ok(commands.iter()
                            .map(|x| add_to_insert(x))
                            .fold("".to_string(), string_concat)),
            DeleteModifier(commands) => 
                Ok(commands.iter()
                            .map(|x| delete_to_insert(x))
                            .fold("".to_string(), string_concat)),
            _ => Err("The AST is empty") 
        }
    }

    fn execute(&self, cmd: &str) -> SimpleContext {
        self.execute_helper(cmd)
    }

}

fn triplet_to_delete(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("DELETE FROM facts WHERE subject='{}' AND link='{}' AND goal='{}'",
            tup.0, tup.1, tup.2)
}

fn delete_to_insert(l: &Language) -> String {
    match l {
        Language::Tri(tri) => triplet_to_delete(&tri),
        _ => String::from("")
    }
}

fn triplet_to_insert(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('{}','{}','{}')",
            tup.0, tup.1, tup.2)
}

fn add_to_insert(l: &Language) -> String {
    match l {
        Language::Tri(tri) => triplet_to_insert(&tri),
        _ => String::from("")
    }
}

fn translate_one_ast<'a>(ast: &'a PredicatAST) -> Result<String, &'a str> {
    match ast {
        Query((get, link, filter)) => Ok(query_to_sql(get, link, filter)),
        AddModifier(commands) => 
            Ok(commands.iter()
                        .map(|x| add_to_insert(x))
                        .fold("".to_string(), string_concat)),
        DeleteModifier(commands) => 
            Ok(commands.iter()
                        .map(|x| delete_to_insert(x))
                        .fold("".to_string(), string_concat)),
        _ => Err("The AST is empty") 
    }
}

fn string_concat(acc: String, x: String) -> String {
    format!("{}{}", acc, x)
}

impl SqliteKnowledge{
    fn execute_helper(&self, s: &str) -> SimpleContext {
        let res = match &s[0..6]  {
            "SELECT" => self.get(s),
            _ => self.modify(s).unwrap()
        }.clone();
        res
    }
}

fn add<'l>(connection: &Connection, elements: &[(&str, Value)]) -> Result<(), sqlite::Error> {
    let sql_query = format!(
        "INSERT or IGNORE INTO facts (subject, link, goal) VALUES ({subject}, {link}, {goal});",
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

fn to_context(hm: HashMap<String, Vec<String>>, columns: Vec<&str>) -> SimpleContext {
    match !hm.is_empty() {
        true => columns.iter().fold(SimpleContext::new(), 
                          |mut acc, x| acc.add_column(x, hm.get(&x.to_string()).unwrap().to_vec())
                    ),
        false => SimpleContext::new()
    }
}

fn query_to_sql(get: &[Var], link: &[Triplet], filter: &[Comp]) -> String {
    let head = format_variables(get);
    let columns = format_triplets(link); // warning, put the result into a parenthese
    let comparisons = format_comparisons(filter);
    format!("{}{}{}", head, columns, comparisons )
}

fn format_triplets(tri: &[Triplet]) -> String {
    if tri == [Triplet::Empty]{
        String::from("facts")
    }
    else {
        let sql_queries = tri.iter()
            .filter_map(|x| Some(triplet_to_sql(x)));
        let queries = sql_queries
            .reduce(|acc, x| format!("({}) natural join ({})", acc, x)).unwrap();
        format!("({})", queries)
    }
}


fn format_variables(vars: &[Var]) -> String {
    // todo: check if it don't lead to problem
    if vars == []{
        String::from("SELECT * FROM ")
    }
    else {
        let extracted_vars = vars.iter()
            .map(|Var(x)| x);
        let string_vars = extracted_vars
            .fold("".to_string(), |acc, x| acc +","+&x[..])
            .chars()
            .skip(1)
            .collect::<String>();
        format!("SELECT {} FROM ",string_vars)
    }
}

fn format_comparisons(comp: &[Comp]) -> String {
    if  comp == [] {
        String::from(";")
    }
    else {
        let comparisons = comp.iter()
            .filter_map(|Comp(c)| Some(c.replace("$","").replace("==","=")));
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
        Triplet::Empty => todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlite_knowledge::extract_columns;
    use crate::sqlite_knowledge::translate_one_ast;

    use super::format_variables;
    use super::Language;
    use super::triplet_to_sql;
    use super::Triplet::*;
    use parser::base_parser::PredicatAST;
    use super::HashMap;
    use super::Var;
    use super::to_context;
    use super::SimpleContext;
    use super::Context;

    #[test]
    fn test_from_triplet_to_sql() {
        assert_eq!(
            triplet_to_sql(&Tvvv("A".to_string(),"B".to_string(),"C".to_string())),
            "SELECT subject AS A,link AS B,goal AS C FROM facts".to_string()
        );
        assert_eq!(
            triplet_to_sql(&Tvwv("A".to_string(),"B".to_string(),"C".to_string())),
            "SELECT subject AS A,goal AS C FROM facts WHERE link='B'"
        );
    }

    #[test]
    fn test_format_variables() {
        assert_eq!(
            format_variables(&vec![Var("X".to_string()),Var("Y".to_string())]),
            "SELECT X,Y FROM "
        );
        assert_eq!(
            format_variables(&vec![Var("X".to_string())]),
            "SELECT X FROM "
        );
    }

    #[test]
    fn test_translate_one_ast_get() {
        assert_eq!(
            translate_one_ast(&PredicatAST::Query((
                    vec![Var("A".to_string())], 
                    vec![Tvww("A".to_string(), "est".to_string(), "mortel".to_string())], 
                    vec![]))).unwrap(),
            "SELECT A FROM (SELECT subject AS A FROM facts WHERE link='est' AND goal='mortel');".to_string());
    }

    #[test]
    fn test_translate_one_ast_add_modifier() {
        assert_eq!(
            translate_one_ast(&PredicatAST::AddModifier(vec![Language::Tri(Twww("pierre".to_string(), "ami".to_string(), "jean".to_string()))])).unwrap(),
            "INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('pierre','ami','jean')".to_string())
    }

    #[test]
    fn test_to_context() {
        let hm = HashMap::from([
                               ("C".to_string(), vec!["alice".to_string()]),
                               ("A".to_string(), vec!["emy".to_string()]),
                               ("B".to_string(), vec!["ami".to_string()]),
        ]);
        let mut sc = SimpleContext::new();
        sc =  sc.add_column("A", vec!["emy".to_string()]);
        sc =  sc.add_column("B", vec!["ami".to_string()]);
        sc =  sc.add_column("C", vec!["alice".to_string()]);

        assert_eq!(
            to_context(hm, vec!["A", "B", "C"]),
            sc);
    }

    #[test]
    fn test_extract_column() {
        assert_eq!(
            extract_columns("SELECT A,B,C FROM (SELECT subject AS A,link AS B,goal AS C FROM facts);"),
            vec!["A", "B", "C"]
            );
    }

}
