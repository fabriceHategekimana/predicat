use cli_table::{Style, Table};
use crate::context_traits::{Context, Var};
use itertools::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum DataFrameError {
    InexistentColumnIn(String, Vec<String>)
}

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
    item.iter().for_each(|(k,v)| hm.add(&Var::format(k), v));
    hm
}

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct DataFrame {
    cells: HashMap<String, Vec<String>>,
    rows: i32,
    columns: i8
}

impl DataFrame {
    pub fn len(&self) -> usize {
       self.cells.iter()
            .next().unwrap_or((&"".to_string(), &vec![])).1.len()
    }

    pub fn empty(&self) -> bool {
        self.rows == 0 && self.columns == 0
    }

    pub fn new() -> Self {
        DataFrame {
            cells: HashMap::new(),
            rows: 0,
            columns: 0
        }
    }

    fn body(t: &[(String, String)]) -> Option<Self> {
        let df = Self::to_dataframe(t);
        match Self::check(&df) {
            true => Some(df.clone()),
            false => None
        }
    } 

    fn to_dataframe(t: &[(String, String)]) -> DataFrame {
        let df = DataFrame { 
            cells: create_hashmap(t.to_vec()),
            rows: 0, 
            columns: 0
        };

        DataFrame {
            rows: Self::nb_rows(&df) as i32,
            columns: Self::nb_columns(&df) as i8,
            ..df
        }
    }

    fn check(df: &DataFrame) -> bool {
        if df.empty() {
            true
        } else {
           let same_size = |x, y| if x == y { x } else { -1 };
           let same = df.get_variables().iter()
               .map(|var| df.get_values(var).unwrap().len() as i32)
               .reduce(same_size)
               .unwrap();
           match same {
               -1 => false,
               _ => true
           }
        }
    }

    fn nb_rows(df: &DataFrame) -> usize {
        if df.empty() {
            return 0 as usize;
        } else {
            let variables = df.get_variables();
            let first_column = variables.iter().next().unwrap();
            df.get_values(&first_column.0).unwrap().len()
        }
    }

    fn nb_columns(df: &DataFrame) -> usize {
        let variables = df.get_variables();
        variables.len()
    }

    fn iter(&self) -> DataFrameIterator {
        DataFrameIterator {
            dataframe: self,
            index: 0,
        }
    }

    fn get_variables(&self) -> Vec<Var>{
        self.cells.keys()
            .map(|var| {
                    Var::new(&var.clone())
                })
            .sorted().unique().collect()
    }

    pub fn get_values(&self, key: &str) -> Result<Vec<String>, DataFrameError> {
        self.cells.get(key).cloned()
            .ok_or(
                DataFrameError::InexistentColumnIn(
                    key.to_string(),
                    self.get_variables().iter().map(|Var(x)| x.to_string()).collect())
                  )
    }

    pub fn get_values2(&self, columns: &[&str]) -> Option<Vec<Vec<String>>> {
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

    fn add_column(&mut self, name: &str, elements: &[&str]) {
        self.cells.insert(name.to_string(),
                          elements.iter().map(|x| x.to_string()).collect());
        self.rows = elements.len() as i32;
        self.columns = self.columns + 1;
    }

    fn is_in_dataframe(&self, key: String) -> bool {
        self.get_variables().iter()
            //.map(Var::without_dollar)
            .any(|x| &x[..] == key)
    }

    fn join(&self, df: Self) -> Self {
       //cells: HashMap<String, Vec<String>>,
       todo!();
    }
}

impl TryFrom<Vec<(String, String)>> for DataFrame {
    type Error = String;

    fn try_from(v: Vec<(String, String)>) -> Result<Self, Self::Error> {
        DataFrame::body(&v).ok_or(String::from("Failed to build Dataframe from Vec<(String, String)>"))
    }
}

impl TryFrom<Vec<Vec<String>>> for DataFrame {
    type Error = String;

    fn try_from(v: Vec<Vec<String>>) -> Result<Self, Self::Error> {
        let vs = v.iter().map(|vec| (vec[0].clone(), vec[1].clone())).collect::<Vec<_>>();
        DataFrame::body(&vs).ok_or(String::from("Failed to build Dataframe from Vec<Vec<String>>"))
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

    pub fn len(&self) -> usize {
        self.tab.len()
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

impl From<DataFrame> for SimpleContext {
    fn from(value: DataFrame) -> SimpleContext {
        SimpleContext { tab: value, cmds: vec![], log: vec![] }
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
    type DataError = DataFrameError;

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

    fn get_values(&self, key: &str) -> Result<Vec<String>, DataFrameError> {
        self.tab.get_values(key)
    }

    fn get_values2(&self, columns: &[&str]) -> Option<Vec<Vec<String>>> {
        self.tab.get_values2(columns)
    }

    fn add_column(&mut self, name: &str, elements: &[&str]) -> SimpleContext{
        self.tab.add_column(name, elements);
        self.clone()
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
        };

        todo!();
    }
    
    fn is_empty(&self) -> bool {
        self.dataframe_len() == 0
    }

    fn is_not_empty(&self) -> bool {
       !self.is_empty() 
    }

    fn get_table(&self) -> HashMap<String, Vec<String>> {
        self.tab.cells.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_dataframe_check_empty() {
        let df = DataFrame::new();
        assert_eq!(DataFrame::check(&df),
                   true);
    }

    #[test]
    fn test_dataframe_check_create_dataframe_from_malformed_vec_of_tuple(){
        let sql_datas = vec![("$A".to_string(), "voila".to_string()),
                         ("$B".to_string(), "truc".to_string()),
                         ("$C".to_string(), "machin".to_string()),
                         ("$C".to_string(), "chose".to_string())];
        let df: Result<DataFrame, String> = sql_datas.try_into();
        assert_eq!(
            df,
            Err(String::from("Failed to build Dataframe from Vec<(String, String)>")));
    }

    #[test]
    fn test_dataframe_check_create_dataframe_from_wellformed_vec_of_tuple(){
        let sql_datas = vec![("$A".to_string(), "voila".to_string()),
                         ("$A".to_string(), "element".to_string()),
                         ("$B".to_string(), "truc".to_string()),
                         ("$B".to_string(), "hey".to_string()),
                         ("$C".to_string(), "machin".to_string()),
                         ("$C".to_string(), "chose".to_string())];
        let df: DataFrame = sql_datas.try_into().unwrap();
        assert_eq!(
            DataFrame::check(&df),
            true);
    }

}

