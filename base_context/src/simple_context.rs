use cli_table::{Style, Table};
use crate::context_traits::{Context, Var};
use itertools::*;
use std::collections::HashMap;

trait Adder {
    fn add(&mut self, k: &str, v: &str);
}

impl Adder for HashMap<String, Vec<String>> {
    fn add(&mut self, k: &str, v: &str) {
        if self.contains_key(k) {
            let val = self.get(k).unwrap().iter()
                        .chain([v.to_string()].iter())
                        .cloned()
                        .collect::<Vec<_>>();
            self.insert(k.to_string(), val);
        } else {
            self.insert(k.to_string(), vec![v.to_string()]);
        }
    }
}

fn create_hashmap(item: Vec<(String,String)>) -> HashMap<String, Vec<String>> {
    let mut hm = HashMap::new();
    item.iter().for_each(|(k,v)| hm.add(k, v));
    hm
}


type ColumnName = String;
type Value = String;

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct DataFrame {
    cells: HashMap<String, Vec<String>>,
    rows: i32,
    columns: i8
}

impl DataFrame {
    fn len(&self) -> usize {
       self.cells.iter()
            .next().unwrap().1.len()
    }

    fn new() -> Self {
        DataFrame {
            cells: HashMap::new(),
            rows: 0,
            columns: 0
        }
    }

    fn body(t: &[(String, String)]) -> Option<Self> {
        Self::check(t)
            .then(||
                    DataFrame { 
                        cells: create_hashmap(t.to_vec()),
                        rows: Self::nb_rows(t),
                        columns: Self::nb_columns(t)
                    })
    } 


    fn check(t: &[(String, String)]) -> bool {
       todo!(); 
       // si le nombre d'élément est proportionnel au nombre de colonnes
       // si chaque colonne a le même nombre d'éléments
    }

    fn nb_rows(t: &[(String, String)]) -> i32 {
        todo!();
    }

    fn nb_columns(t: &[(String, String)]) -> i8 {
        todo!();
    }

    fn iter(&self) -> DataFrameIterator {
        DataFrameIterator {
            dataframe: self,
            index: 0,
        }
    }

    fn get_variables(&self) -> Vec<Var>{
        self.iter()
            .map(|(var, _val)| {
                    Var::new(&var.clone())
                })
            .sorted().unique().collect()
    }

    fn get_values(&self, key: &str) -> Option<Vec<String>> {
        match self.is_in_dataframe(key.to_string()) {
            true => Some(self.iter().filter(|x| x.0.clone() == key).map(|x| x.1.clone()).collect::<Vec<String>>()),
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
        let new_tab = self.iter().chain(tab).map(|x| x.clone()).collect::<Vec<_>>();
        SimpleContext::try_from(new_tab).unwrap()
    }

    fn is_in_dataframe(&self, key: String) -> bool {
        self.get_variables().iter().map(Var::without_dollar).any(|x| &x[..] == key)
    }
}


struct DataFrameIterator<'a> {
    dataframe: &'a DataFrame,
    index: usize,
}

impl<'a> Iterator for DataFrameIterator<'a> {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.dataframe.len() {
            //let result = &self.dataframe.cells.get(self.index).unwrap();
            // get the ith index of the dataframe for each column
            let result = self.dataframe.get_variables()
                .iter().map(|var| self.dataframe.cells.get(&var.0).unwrap()[self.index].clone())
                .collect::<Vec<_>>();
            self.index += 1;
            Some(result.clone())
        } else {
            None
        }
    }
}

impl TryInto<DataFrame> for Vec<(String, String)> {
    type Error = String;
    fn try_into(self) -> Result<DataFrame, Self::Error> {
        DataFrame::body(&self).ok_or("Wasn't able to convert to dataframe".to_string())
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
            let variables = self.get_variables().iter()
                .map(Var::without_dollar).collect::<Vec<String>>();
            let body = self.get_variables().iter()
                .map(|x| self.get_values(&x.without_dollar()).unwrap_or(vec![]))
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
    body.iter().map(|x| x.iter().nth(num).unwrap_or(&"".to_string()).clone()).collect()
}

impl TryFrom<Vec<(String, String)>> for SimpleContext {
    type Error = String;

    fn try_from(v: Vec<(String, String)>) -> Result<Self, Self::Error> {
        match DataFrame::body(&v) {
            Some(df) => Ok(SimpleContext { tab: df, cmds: vec![], log: vec![]}),
            _ => Err("We weren't able to convert the list of tuple to a context".to_string())
        }
    }
}


impl From<Vec<[&str; 3]>> for SimpleContext {
    fn from(v: Vec<[&str; 3]>) -> SimpleContext {
       v.iter()
        .flat_map(|x| [("subject".to_string(), x[0].to_string()),
                        ("link".to_string(), x[1].to_string()),
                        ("goal".to_string(), x[2].to_string())])
        .collect::<Vec<(String, String)>>()
        .try_into().expect("The dataframe is malformed")
    }
}

impl Context for SimpleContext {

    type FellowContext = SimpleContext;

    fn new() -> SimpleContext {
        SimpleContext{
            tab: DataFrame::new(),
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
        self.tab.is_in_dataframe(key)
    }

    fn dataframe_len(&self) -> usize {
        self.tab.len()
    }

    fn join(&self, c2: SimpleContext) -> SimpleContext {
        let vec_tab = self.get_tab()
                    .iter()
                    .chain(c2.get_tab().iter())
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
            tab: vec_tab.try_into().unwrap(),
            cmds: vec_cmds,
            log: vec_log,
        }
    }
    
    fn is_empty(&self) -> bool {
        self.dataframe_len() == 0
    }

    fn is_not_empty(&self) -> bool {
       !self.is_empty() 
    }

    fn get_table(&self) -> Vec<(String, String)> {
        self.tab.cells.clone()
    }
}
