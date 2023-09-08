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
use parser::base_parser::PredicatAST::{Query, AddModifier, DeleteModifier, Empty, Rule};

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

static CREATE_RULES : &str = "CREATE TABLE IF NOT EXISTS rules(
                    'id' INTEGER PRIMARY KEY AUTOINCREMENT,
                    'name' TEXT,
                    'event' TEXT, 
                    'modifier' TEXT, 
                    'subject' TEXT, 
                    'link' TEXT, 
                    'goal' TEXT, 
                    'command' TEXT,
                    'backed_command');
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
    connection: Connection,
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
            connection: sqlite::open("data.db").unwrap(),
        };
        let _ = knowledge.modify(CREATE_FACTS);
        let _ = knowledge.modify(CREATE_RULES);
        knowledge
    }

    fn clear(&self) {
        let _ = self.connection.execute("DELETE FROM facts");
        let _ = self.connection.execute("DELETE FROM rules");
    }

    fn get(&self, cmd: &str) -> SimpleContext {
        let query = cmd;
        let mut v: Vec<(String, String)> = vec![];
        let _ = self.connection.iterate(query, |sqlite_couple| {
            for couple in sqlite_couple.iter() {
                v.push((couple.0.to_string(),
                        couple.1.unwrap_or("").to_string()));
            }
            true
        });
        SimpleContext::from(&v)
    }

    fn get_all(&self) -> SimpleContext {
        self.get("SELECT A,B,C from (SELECT subject as A, link as B, goal as C FROM facts)")
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
                            .map(|x| triplet_to_insert(x))
                            .fold("".to_string(), string_concat)),
            DeleteModifier(commands) => 
                Ok(commands.iter()
                            .map(|x| triplet_to_insert(x))
                            .fold("".to_string(), string_concat)),
                Rule(a, (b, c), (cmd, ast)) => {
                    let scmd = match &cmd[0..3] { 
                        "get" => self.translate(ast).unwrap().replace("'", "%single_quote%"),
                        _ => cmd.clone()};
                    let (t1, t2, t3) = c.to_tuple_with_variable();
                    Ok(format!("%rule%%|%{:?}%|%{:?}%|%{}%|%{}%|%{}%|%{}%|%{}",
                               a, b, t1, t2, t3, cmd, scmd))
                            },
            _ => Err("The AST is empty") 
        }
    }

    fn execute(&self, s: &str) -> SimpleContext {
        let res = match &s[0..6]  {
            "SELECT" => self.get(s),
            "%rule%" => self.store_rule(s),
            _ => self.modify(s).unwrap()
        }.clone();
        res
    }

    fn is_invalid(&self, cmd: &PredicatAST) -> bool {
        match cmd {
            PredicatAST::Rule(a, (mo, tri), cmd) => {
                let (t1, t2, t3) = tri.to_tuple_with_variable();
                let select = format!("SELECT * FROM Rules where modifier = {:?} subject = {:?} or link = {:?} or goal = {:?}",
                        mo, t1, t2, t3);
                match self.get(&select).get_values("backed_command") {
                    None => false,
                    Some(v) => v.iter()
                        .map(|x| x.replace("%singlequote%", "'"))
                        .any(|cmd| self.get(&cmd).is_not_empty())
                }
            },
            _ => false
        }
    }

    fn get_command_from_triplet(&self, modifier: &str, tri: &Triplet) -> Vec<String> {
        let (sub, lin, goa) = tri.to_tuple();
        let select = format!("SELECT command FROM rules where modifier='{}' AND event='infer' OR subject='{}' OR link='{}' OR goal='{}'", modifier, sub, lin, goa);
        self.get(&select).get_values("command").unwrap_or(vec![]) 
    }

    fn get_commands_from(&self, cmd: &PredicatAST) -> Vec<String> {
        match cmd {
            PredicatAST::AddModifier(v_of_tri) => v_of_tri.iter()
                .flat_map(|x| self.get_command_from_triplet("add", x))
                .collect::<Vec<_>>(),
            PredicatAST::DeleteModifier(v_of_tri) => v_of_tri.iter()
                .flat_map(|x| self.get_command_from_triplet("delete", x))
                .collect::<Vec<_>>(),
            _ => vec![]
        }
    }

}

fn triplet_to_delete(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("DELETE FROM facts WHERE subject='{}' AND link='{}' AND goal='{}'",
            tup.0, tup.1, tup.2)
}

fn triplet_to_insert(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('{}','{}','{}')",
            tup.0, tup.1, tup.2)
}

fn translate_one_ast<'a>(ast: &'a PredicatAST) -> Result<String, &'a str> {
    match ast {
        Query((get, link, filter)) => Ok(query_to_sql(get, link, filter)),
        AddModifier(commands) => 
            Ok(commands.iter()
                        .map(|x| triplet_to_insert(x))
                        .fold("".to_string(), string_concat)),
        DeleteModifier(commands) => 
            Ok(commands.iter()
                        .map(|x| triplet_to_insert(x))
                        .fold("".to_string(), string_concat)),
        _ => Err("The AST is empty") 
    }
}

fn string_concat(acc: String, x: String) -> String {
    format!("{}{}", acc, x)
}

impl SqliteKnowledge{

    fn get_vec(&self, cmd: &str) -> Vec<(String, String)> {
        let query = cmd;
        let mut v: Vec<(String, String)> = vec![];
        let _ = self.connection.iterate(query, |sqlite_couple| {
            for couple in sqlite_couple.iter() {
                v.push((couple.0.to_string(),
                        couple.1.unwrap_or("").to_string()));
            }
            true
        });
        v.clone()
    }

    fn store_rule(&self, s: &str) -> SimpleContext {
        let values = s.split("%|%").collect::<Vec<_>>();
        let cmd = format!("INSERT INTO rules (event, modifier, subject, link, goal, command, backed_command) VALUES (\'{}\', \'{}\', \'{}\', \'{}\', \'{}\', \'{}\', \'{}\')",
                    values[1], values[2], values[3], values[4], values[5], values[6], values[7]);
        match self.connection.execute(cmd) {
           Err(e) => {dbg!(e); SimpleContext::new()}
           _ => SimpleContext::new(),
        }
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
                          |mut acc, x| { 
                              acc.add_column(
                                  x,
                                  &hm.get(&x[..]).unwrap_or(&vec![])
                                  .iter()
                                  .map(|x| &x[..])
                                  .collect::<Vec<_>>()
                                  )
                          }
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
        Triplet::Empty => String::from("")
    }
}


#[cfg(test)]
mod tests {
    use crate::sqlite_knowledge::SqliteKnowledge;
    use crate::sqlite_knowledge::extract_columns;
    use crate::sqlite_knowledge::translate_one_ast;
    use super::Knowledgeable;

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
    use super::*;

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
            translate_one_ast(&PredicatAST::AddModifier(vec![Twww("pierre".to_string(), "ami".to_string(), "jean".to_string())])).unwrap(),
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
        sc =  sc.add_column("A", &["emy"]);
        sc =  sc.add_column("B", &["ami"]);
        sc =  sc.add_column("C", &["alice"]);

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

    #[test]
    fn test_extract_column2() {
        assert_eq!(
            extract_columns("SELECT $A,$B,$C FROM (SELECT subject AS A,link AS B,goal AS C FROM facts);"),
            vec!["$A", "$B", "$C"]
            );
    }

    #[test]
    fn test_to_context2() {
        let mut hm: HashMap<String, Vec<String>> = HashMap::new();
        hm.insert("A".to_owned(), vec!["socrate".to_owned()]);
        hm.insert("B".to_owned(), vec!["est".to_owned()]);
        hm.insert("C".to_owned(), vec!["mortel".to_owned()]);
        let new_context = to_context(hm, vec!["A", "B", "C"]);
        assert_eq!(
            new_context.get_values("A"),
            Some(vec!["socrate".to_string()])
                  );
    }


}
