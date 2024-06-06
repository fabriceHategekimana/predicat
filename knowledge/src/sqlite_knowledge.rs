#![allow(dead_code, unused_variables, unused_imports)]

use sqlite::{
        Connection,
        Value,
        Statement,
};

//use crate::parser::parse_command;
use base_context::context_traits::{Context, Var};
use base_context::simple_context::SimpleContext;
use metaprogramming::substitute_variables;
use std::collections::HashMap;
use super::Knowledgeable;
use crate::base_knowledge::{Command, FactManager, Cache, RuleManager};
use parser::soft_predicat;
use parser::base_parser::PredicatAST;
use parser::base_parser::PredicatAST::{Query, AddModifier, DeleteModifier, Empty, Infer};
use parser::parse_command;
use parser::base_parser::Language;
use parser::base_parser::Language::Element;
use parser::base_parser::Language::Tri;
use parser::base_parser::Comp;
use parser::base_parser::Triplet::*;
use parser::base_parser::Triplet;
use itertools::izip;
use itertools::Itertools;
use serial_test::serial;

//pub struct Sql(String);

pub enum Sql {
    Query(String),
    Rule(String),
    Modify(String)
}

impl Into<Sql> for String {
    fn into(self) -> Sql {
        match &self[0..6]  {
            "SELECT" => Sql::Query(self.to_string()),
            "%rule%" => Sql::Rule(self.to_string()),
            _ => Sql::Modify(self.to_string())
        }
    }
}

impl Into<Sql> for &str {
    fn into(self) -> Sql {
        String::into(self.to_string())
    }
}

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
                    'modifier' TEXT, 
                    'subject' TEXT, 
                    'link' TEXT, 
                    'goal' TEXT, 
                    'command' TEXT,
                    'backed_command');
                    ";

static CREATE_CACHE : &str = "CREATE TABLE IF NOT EXISTS cache(
                            'command' TEXT); 
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



impl Command for SqliteKnowledge {
    type Language = Sql;

    fn get(&self, cmd: &str) -> SimpleContext {
        let mut v: Vec<(String, String)> = vec![];
        let _ = self.connection.iterate(cmd, |sqlite_couple| {
            for couple in sqlite_couple.iter() {
                v.push((couple.0.to_string(),
                        couple.1.unwrap_or("").to_string()));
            }
            true
        });
        SimpleContext::from(v)
    }

    fn get_all(&self) -> SimpleContext {
        self.get(&"SELECT A,B,C from (SELECT subject as A, link as B, goal as C FROM facts)")
    }

    fn modify(&self, cmd: &str) -> Result<SimpleContext, &str> {
        match self.connection.execute(cmd) {
            Ok(r) => Ok(SimpleContext::new()),
            Err(r) => {println!("r: {:?}", r); Err("An error occured with the sqlite database")}
        }
    }


    fn translate<'a>(&'a self, ast: &PredicatAST) -> Result<Vec<Sql>, &str> {
        match ast {
            Query((get, link, filter)) => Ok(vec![query_to_sql(get, link, filter).into()]),
            AddModifier(commands) => 
                Ok(vec![commands.iter()
                            .map(|x| triplet_to_insert(x))
                            .fold("".to_string(), string_concat)
                            .into()]),
            DeleteModifier(commands) => 
                Ok(vec![commands.iter()
                            .map(|x| triplet_to_delete(x))
                            .fold("".to_string(), string_concat)
                            .into()]),
            Infer((b, c), pre, cmd) => {
                    // TODO : perhaps remove the match (isn't used yet)
                    let scmd = match &cmd[0..3] { 
                        "get" => self.translate(ast).unwrap().iter()
                                .map(|x| {
                                    if let Sql::Rule(rule) = x {
                                        Some(rule.replace("'", "%single_quote%"))
                                    } else { None }.unwrap() })
                                .collect(),
                        _ => cmd.clone()};
                    let res = c.iter().map(|x| x.to_tuple_with_variable())
                        .map(|(t1, t2, t3)| {
                            format!("%rule%%|%{}%|%{}%|%{}%|%{}%|%{}%|%{}",
                                       b.get_string(), t1, t2, t3, pre, cmd).into()
                        }).collect::<Vec<_>>();
                        Ok(res)
                            },
            _ => Err("The AST is empty") 
        }
    }

    fn execute(&self, s: &Sql) -> SimpleContext {
        let res = match s  {
            Sql::Query(q) => self.get(q),
            Sql::Rule(r) => self.store_rule(r),
            Sql::Modify(m) => self.modify(m).unwrap()
        }.clone();
        res
    }

    fn is_invalid(&self, cmd: &PredicatAST) -> bool {
        match cmd {
            PredicatAST::Infer((mo, tri), pre, cmd) => {
                tri.iter().map(|x| x.to_tuple_with_variable())
                    .map(|(t1, t2, t3)| {
                        let select = format!("SELECT * FROM Rules where modifier = {:?} subject = {:?} or link = {:?} or goal = {:?}",
                        mo, t1, t2, t3);
                    match self.get(&select).get_values("backed_command") {
                        None => false,
                        Some(v) => v.iter()
                            .map(|x| x.replace("%singlequote%", "'"))
                            .any(|cmd| self.get(&cmd).is_not_empty())
                        }
                    }).any(|x| x)
            },
            _ => false
        }
    }


    fn infer_command_from_triplet(&self, modifier: &str, tri: &Triplet) -> Vec<String> {
        let (sub, lin, goa) = tri.to_tuple();
        let select = format!("SELECT * FROM rules where modifier='{}' AND (subject='{}' OR link='{}' OR goal='{}')", modifier, sub, lin, goa);
        let rules = self.get(&select);
        let dataframe_of_variables = rules.get_values2(&["modifier, subject", "link", "goal"])
            .unwrap().iter().map(|x| x.into_iter().collect_tuple().unwrap()) // to tuple
            .map(|(modi, subj, link, goal)| unify_triplet((&sub, &lin, &goa), (&subj, &link, &goal)))
            .reduce(|context1, context2| context1.join(context2))
            .unwrap_or(SimpleContext::new());

        rules.get_values("command").unwrap_or(vec![]).iter()
            .map(|cmd| change_variables(cmd, &dataframe_of_variables))
            .collect()
    }

    fn infer_commands_from(&self, cmd: &PredicatAST) -> Vec<String> {
        match cmd {
            PredicatAST::AddModifier(v_of_tri) => v_of_tri.iter()
                .flat_map(|x| self.infer_command_from_triplet("add", x))
                .collect::<Vec<_>>(),
            PredicatAST::DeleteModifier(v_of_tri) => v_of_tri.iter()
                .flat_map(|x| self.infer_command_from_triplet("delete", x))
                .collect::<Vec<_>>(),
            _ => vec![]
        }
    }

}


impl FactManager for SqliteKnowledge {
    fn save_facts(&self, modifier: &str, subject: &str, link: &str, goal: &str) {
        let query = format!("INSERT INTO cache (command) VALUES ('{} {} {} {}')",
            modifier, subject, link, goal);
        let res = &self.connection.execute(query);
    }

    fn clear_facts(&self) {
        let _ = self.connection.execute("DELETE FROM facts");
    }

}

impl Cache for SqliteKnowledge {
    fn clear_cache(&self) {
        let _ = self.connection.execute("DELETE FROM cache");
    }

    fn in_cache(&self, cmd: &PredicatAST) -> bool {
        let mut res: bool = false;
        let query = format!("SELECT * FROM cache WHERE command = '{}'", String::from(cmd.clone()));
        let _ = self.connection.iterate(query, |sqlite_couple| {
            if sqlite_couple.iter().len() > 0 {
                res = true;
            }
            true
        });
        res
    }

    fn store_to_cache(&self, modifier: &PredicatAST) -> PredicatAST {
        let query = format!("INSERT INTO cache (command) VALUES ('{}')",
            String::from(modifier.clone()));
        let res = &self.connection.execute(query);
        modifier.clone()
    }

}

impl RuleManager for SqliteKnowledge {
    fn clear_rules(&self) {
        let _ = self.connection.execute("DELETE FROM rules");
    }

    fn store_rule(&self, s: &str) -> SimpleContext {
        let values = s.split("%|%").collect::<Vec<_>>();
        let cmd = format!("INSERT INTO rules (modifier, subject, link, goal, command, backed_command) VALUES (\'{}\', \'{}\', \'{}\', \'{}\', \'{}\', \'{}\')",
                    values[1], values[2], values[3], values[4], values[5], values[6]);
        match self.connection.execute(cmd) {
           Err(e) => {dbg!(e); SimpleContext::new()}
           _ => SimpleContext::new(),
        }
    }

    fn get_rules(&self) -> Vec<String> {
        let query = "SELECT modifier, subject, link, goal, command, backed_command FROM rules;".to_string();
        let mut v: Vec<String> = vec![];
        let _ = self.connection.iterate(query, |sqlite_couple| {
            for couple in sqlite_couple.iter() {
                v.push(couple.1.unwrap_or("").to_string());
            }
            true
        });
        v
    }
}


impl Knowledgeable for SqliteKnowledge {
    fn new() -> SqliteKnowledge {
        let knowledge = SqliteKnowledge {
            connection: sqlite::open("data.db").unwrap(),
        };
        let _ = knowledge.modify(&CREATE_FACTS);
        let _ = knowledge.modify(&CREATE_RULES);
        let _ = knowledge.modify(&CREATE_CACHE);
        knowledge
    }
}

fn unify_triplet((sub1, lin1, goa1): (&str, &str, &str), (sub2, lin2, goa2): (&str, &str, &str)) -> SimpleContext {
    let res = [(sub1, sub2), (lin1, lin2), (goa1, goa2)]
        .iter()
        .flat_map(|(el1, el2)| match is_variable(el2) { true => Some((el2.to_string(), el1.to_string())), false => None })
        .collect::<Vec<_>>();
         SimpleContext::from(res)
}

fn substitute_variable(var: &Var, val:&str, cmd: &str) -> String {
    cmd.replace(&var.0, &format!("'{}'", val)).to_string()
}

fn change_variables(cmd: &str, context: &SimpleContext) -> String {
    let cmds = (0..(context.dataframe_len()))
        .map(|_| cmd.to_string()).collect::<Vec<_>>();

    context.get_variables().iter()
        .fold(cmds, |commands, var| 
             context.get_values(&var.0)
             .unwrap_or(vec![]).iter().zip(commands.iter())
             .map(|(val, cmd)| substitute_variable(var, val, cmd))
             .collect())
        .get(0).unwrap_or(&"".to_string()).clone()
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
    format!("{}{}{}", head, columns, comparisons)
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
        Teee(a,b,c) => 
            format!("SELECT subject,link,goal FROM facts WHERE subject='{}' AND link='{}' AND goal='{}'",a,b,c),
        Tvee(a,b,c) => 
            format!("SELECT subject AS {} FROM facts WHERE link='{}' AND goal='{}'",a,b,c),
        Teve(a,b,c) => 
            format!("SELECT link AS {} FROM facts WHERE subject='{}' AND goal='{}'",b,a,c),
        Teev(a,b,c) => 
            format!("SELECT goal AS {} FROM facts WHERE subject='{}' AND link='{}'",c,a,b),
        Tvve(a,b,c) => 
            format!("SELECT subject AS {},link AS {} FROM facts WHERE goal='{}'",a,b,c),
        Tvev(a,b,c) => 
            format!("SELECT subject AS {},goal AS {} FROM facts WHERE link='{}'",a,c,b),
        Tevv(a,b,c) => 
            format!("SELECT link AS {},goal AS {} FROM facts WHERE subject='{}'",b,c,a),
        Tvvv(a,b,c) => 
            format!("SELECT subject AS {},link AS {},goal AS {} FROM facts",a,b,c),
        Triplet::Empty => String::from(""),
        TNeee(a,b,c) => 
            format!("SELECT subject AS {}, link AS {},goal AS {} FROM facts RIGHT JOIN facts t2 ON t1.subject = t2.subject AND t1.link = t2.link AND t1.goal = t2.goal",a,b,c),
        TNvee(a,b,c) => 
            format!("SELECT subject AS {} FROM facts WHERE subject NOT IN (SELECT subject AS {} FROM facts WHERE link='{}' AND goal='{}')", a,a,b,c),
        TNeve(a,b,c) => 
            format!("SELECT link AS {} FROM facts WHERE link NOT IN (SELECT link AS {} FROM facts WHERE subject='{}' AND goal='{}')", b,b,a,c),
        TNeev(a,b,c) => 
            format!("SELECT goal AS {} FROM facts WHERE link NOT IN (SELECT goal AS {} FROM facts WHERE subject='{}' AND link='{}')",c,c,a,b),
        TNvve(a,b,c) => 
            format!("SELECT subject AS {},link AS {} FROM facts t1 WHERE goal='{}' RIGHT JOIN facts t2 ON t1.subject = t2.subject AND t1.link = t2.link",a,b,c),
        TNvev(a,b,c) => 
            format!("SELECT subject AS {},goal AS {} FROM facts WHERE link='{}' RIGHT JOIN facts t2 ON t1.subject = t2.subject AND t1.goal = t2.goal",a,c,b),
        TNevv(a,b,c) => 
            format!("SELECT link AS {},goal AS {} FROM facts WHERE subject='{}' RIGHT JOIN facts t2 ON t1.link = t2.link AND t1.goal = t2.goal",b,c,a),
        TNvvv(a,b,c) => String::from("SELECT * FROM facts HAVING 1 = 0"),
    }
}

fn is_variable(s: &str) -> bool {
    &s[0..1] == "$"
}

fn extract_substitution_list(triplet: Triplet, tri_param: &[&str]) -> Vec<(String, String)> {
    match triplet {
        Triplet::Tvvv(s, l, g) => [s, l, g].into_iter()
            .zip(tri_param.iter())
            .filter(|(val, ele)| is_variable(ele))
            .map(|(val, ele)| (val.to_string(), ele.to_string()))
            .collect::<Vec<_>>(),
        _ =>vec![]
    }
}


fn substitute_command_string(triplet: Triplet, tri_param: &[&str], cmd: &str) -> String {
    extract_substitution_list(triplet, tri_param).iter()
        .fold(cmd.to_string(), |command, (val, ele)| command.to_string().replace(ele, val)).to_string()
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
            triplet_to_sql(&Tvev("A".to_string(),"B".to_string(),"C".to_string())),
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
                    vec![Tvee("A".to_string(), "est".to_string(), "mortel".to_string())], 
                    vec![]))).unwrap(),
            "SELECT A FROM (SELECT subject AS A FROM facts WHERE link='est' AND goal='mortel');".to_string());
    }

    #[test]
    fn test_translate_one_ast_add_modifier() {
        assert_eq!(
            translate_one_ast(&PredicatAST::AddModifier(vec![Teee("pierre".to_string(), "ami".to_string(), "jean".to_string())])).unwrap(),
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

    #[test]
    fn test_substitute_command_string() {
        assert_eq!(
            substitute_command_string(
                Triplet::Tvvv("pierre".to_string(), "ami".to_string(), "emy".to_string()), 
                &["$A", "ami", "$B"],
                "add $B ami $A"),
                "add emy ami pierre" 
                );
    }

    #[test]
    fn test_match_triplet() {
        let mut context = SimpleContext::new();
        context = context.add_column("$A", &["pierre"]);
        context = context.add_column("$B", &["emi"]);

        assert_eq!(
            unify_triplet(("pierre", "ami", "emi"), ("$A", "ami", "$B")),
            context   
        );
    }

    #[test]
    fn test_substitute_variable() {
        assert_eq!(
            substitute_variable("$A", "pierre", "add $A ami $B"),
            "add pierre ami $B".to_string());
    }

    #[test]
    fn test_change_variable() {
        let mut context = SimpleContext::new();
        context = context.add_column("$A", &["pierre"]);
        context = context.add_column("$B", &["emy"]);
        assert_eq!(
            change_variables("add $B ami $A", &context),
            "add emy ami pierre");
    }

}
