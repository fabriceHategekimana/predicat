use cli_table::{Style, Table};
use crate::context_traits::{Context, Var};
use itertools::*;

type ColumnName = String;
type Value = String;

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct DataFrame(Vec<(ColumnName, Value)>);

impl DataFrame {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get_variables(&self) -> Vec<Var>{
        self.clone().map(|x| Var::new(&x.0.clone()).unwrap()).sorted().unique().collect()
    }

    fn get_values(&self, key: &str) -> Option<Vec<String>> {
        match self.is_in_context(key.to_string()) {
            true => Some(self.clone().filter(|x| x.0.clone() == key).map(|x| x.1.clone()).collect::<Vec<String>>()),
            _ => None
        }
    }

    fn get_values2(&self, columns: &[&str]) -> Option<Vec<Vec<String>>> {
        let res = columns.iter()
            .flat_map(|c| self.get_values(c))
            .collect::<Vec<_>>();
        if res.len() < columns.len() {
            None
        } else {
            Some((0..(res[0].len()))
                .map(|index| res.iter().map(|x| x[index].clone()).collect::<Vec<_>>())
                .collect())
        }
    }

    fn add_column(&mut self, name: &str, elements: &[&str]) -> SimpleContext {
        let tab = elements.iter()
                          .map(|x| (name.to_string(), x.to_string()))
                          .collect::<Vec<(String, String)>>();
        let new_tab = self.chain(tab).map(|x| x.clone()).collect::<Vec<_>>();
        SimpleContext::from(new_tab)
    }

    fn is_in_context(&self, key: String) -> bool {
        self.get_variables().iter().any(|x| &x.0[..] == key)
    }
}

impl Iterator for DataFrame {
    type Item = (String, String);

    fn next(&mut self) ->Option<Self::Item> {
        self.0.iter().next().cloned()
    }
}


impl Into<DataFrame> for Vec<(String, String)> {
    fn into(self) -> DataFrame {
       DataFrame(self)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct SimpleContext {
    pub tab: DataFrame,
    pub cmds: Vec<String>,
    pub log: Vec<String>
}

impl SimpleContext {
    pub fn join_contexts(ctx1: SimpleContext, ctx2: SimpleContext) -> SimpleContext {
        ctx1.join(ctx2)
    }

    pub fn has_commands(&self) -> bool {
        self.cmds != vec![] as Vec<String>
    }

    pub fn has_error(&self) -> bool {
        self.log != vec![] as Vec<String>
    }



pub fn display(&self) {
    // TODO : display error if any and call revert back in the back
    match self.dataframe_len() {
        x if x > 0 => {
        let variables = self.get_variables();
        let body = self.get_variables().iter()
            .map(|x| self.get_values(&x.0).unwrap())
            .collect::<Vec<_>>();
        let table = (0..self.dataframe_len()).map(|x| get_line(x, &body))
            .table()
            .title(variables)
            .bold(true)
            .display()
            .unwrap();
            println!("{}", table);
        }
        _ => println!("EMPTY")
    }
}

    pub fn get_tab(&self) -> DataFrame {
        self.tab.clone()
    }
}

fn get_line(num: usize, body: &[Vec<String>]) -> Vec<String> {
    body.iter().map(|x| x[num].clone()).collect()
}

impl From<Vec<(String, String)>> for SimpleContext {
    fn from(v: Vec<(String, String)>) -> SimpleContext {
        SimpleContext { tab: v.into(), cmds: vec![], log: vec![] }
    }
}


impl From<Vec<[&str; 3]>> for SimpleContext {
    fn from(v: Vec<[&str; 3]>) -> SimpleContext {
       v.iter()
        .flat_map(|x| [("subject".to_string(), x[0].to_string()),
                        ("link".to_string(), x[1].to_string()),
                        ("goal".to_string(), x[2].to_string())])
        .collect::<Vec<(String, String)>>()
        .into()
    }
}

fn join_vec<T: Clone>(v1: Vec<T>, v2: Vec<T>) -> Vec<T> {
        v1.iter()
        .chain(v2.iter())
        .map(|x| x.clone())
        .collect()
}

impl Context for SimpleContext {

    type FellowContext = SimpleContext;

    fn new() -> SimpleContext {
        SimpleContext{
            tab: DataFrame(vec![]),
            cmds: vec![],
            log: vec![]
        }
    }

    fn get_variables(&self) -> Vec<Var> {
        self.tab.get_variables()
    }

    fn get_values(&self, key: &str) -> Option<Vec<String>>{
        self.tab.get_values(key)
    }

    fn get_values2(&self, columns: &[&str]) -> Option<Vec<Vec<String>>> {
        self.tab.get_values2(columns)
    }

    fn add_column(&mut self, name: &str, elements: &[&str]) -> Self{
        self.tab.add_column(name, elements)
    }

    fn is_in_context(&self, key: String) -> bool {
        self.tab.is_in_context(key)
    }

    fn dataframe_len(&self) -> usize {
        self.tab.len()
    }

    fn join(&self, c2: SimpleContext) -> SimpleContext {
        let vec_tab = self.get_tab()
                    .chain(c2.get_tab())
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();
        let vec_cmds = self.cmds
                    .iter()
                    .chain(c2.cmds.iter())
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();
        let vec_log = self.log
                    .iter()
                    .chain(c2.log.iter())
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

        SimpleContext {
            tab: vec_tab.clone().into(),
            cmds: vec_cmds,
            log: vec_log,
        }
    }
    
    fn is_empty(&self) -> bool {
        self.dataframe_len() > 0
    }

    fn is_not_empty(&self) -> bool {
       !self.is_empty() 
    }

    fn get_table(&self) -> Vec<(String, String)> {
        self.tab.0.clone()
    }
}
