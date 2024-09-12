#![allow(dead_code, unused_variables, unused_imports, unreachable_code, unused_assignments)]
use std::collections::HashMap;
use crate::simple_context::Adder;
use parser::Var;
use itertools::Itertools;


#[derive(Debug)]
pub enum DataFrameError {
    InexistentColumnIn(String, Vec<String>),
    MalFormedHashMap
}

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct DataFrame {
    cells: HashMap<String, Vec<String>>,
    rows: i32,
    columns: i8
}

impl DataFrame {
    pub fn new() -> Self {
        DataFrame {
            cells: HashMap::new(),
            rows: 0,
            columns: 0
        }
    }

    pub fn len(&self) -> usize {
       self.cells.iter()
            .next().unwrap_or((&"".to_string(), &vec![])).1.len()
    }

    pub fn empty(&self) -> bool {
        let var1 = self.rows == 0;
        let var2 = self.columns == 0;
        let var3 = var1 && var2;
        var3
    }


    pub fn body(t: &[(String, String)]) -> Option<Self> {
        let df = Self::to_dataframe(t);
        Self::check(&df).then_some(df.clone())
    } 

    fn to_dataframe(t: &[(String, String)]) -> DataFrame {
        let df = DataFrame { 
            cells: Self::create_hashmap(t.to_vec()),
            rows: 0, 
            columns: 0
        };

        DataFrame {
            rows: Self::nb_rows(&df) as i32,
            columns: Self::nb_columns(&df) as i8,
            ..df
        }
    }

    pub fn check(df: &DataFrame) -> bool {
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

    pub fn nb_rows(&self) -> usize {
        if self.get_variables().len() == 0 {
            return 0 as usize;
        } else {
            let variables = self.get_variables();
            let first_column = variables.iter().next().unwrap();
            self.get_values(&first_column.0).unwrap().len()
        }
    }

    pub fn nb_columns(&self) -> usize {
        let variables = self.get_variables();
        variables.len()
    }

    pub fn iter(&self) -> DataFrameIterator {
        DataFrameIterator {
            dataframe: self,
            index: 0,
        }
    }

    pub fn get_variables(&self) -> Vec<Var>{
        self.cells.keys()
            .map(|var| {
                    Var::new(&var.clone())
                })
            .sorted().unique().collect()
    }

    pub fn get_values(&self, key: &str) -> Result<Vec<String>, DataFrameError> {
        let key = if &key[..1] != "$" { format!("${}", key) } else { key.to_string() };
        self.cells.get(&key).cloned()
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

    pub fn add_column(&mut self, name: &str, elements: &[&str]) {
        self.cells.insert(name.to_string(),
                          elements.iter().map(|x| x.to_string()).collect());
        self.rows = elements.len() as i32;
        self.columns = self.columns + 1;
    }

    pub fn is_in_dataframe(&self, key: String) -> bool {
        self.get_variables().iter()
            //.map(Var::without_dollar)
            .any(|x| &x[..] == key)
    }

    fn join(&self, _df: &Self) -> Self {
       //cells: HashMap<String, Vec<String>>,
       todo!();
    }

    fn create_hashmap(item: Vec<(String,String)>) -> HashMap<String, Vec<String>> {
        let mut hm = HashMap::new();
        item.iter().for_each(|(k,v)| hm.add(&Var::format(k), v));
        hm
    }

    pub fn get_cells(&self) -> HashMap<String, Vec<String>> {
       self.cells.clone()
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

impl TryFrom<HashMap<String, Vec<String>>> for DataFrame {
    type Error = DataFrameError;

    fn try_from(hm: HashMap<String, Vec<String>>) -> Result<Self, Self::Error> {
        let df = DataFrame { cells: hm, ..DataFrame::new() };
        let df = DataFrame {
            rows: df.len() as i32,
            columns: df.get_variables().len() as i8,
            ..df
        };
        DataFrame::check(&df).then_some(df).ok_or(DataFrameError::MalFormedHashMap)
    }
}


pub struct DataFrameIterator<'a> {
    dataframe: &'a DataFrame,
    index: usize,
}

impl<'a> Iterator for DataFrameIterator<'a> {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.dataframe.len() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataframe_get_variables(){
        let df: DataFrame = vec![("A".to_string(), "hey".to_string()), 
                      ("B".to_string(), "wow".to_string()),
                      ("C".to_string(), "truc".to_string())].try_into().unwrap();
        assert_eq!(
            df.get_variables(),
            [Var("$A".to_string()), Var("$B".to_string()), Var("$C".to_string())]);
    }

    #[test]
    fn test_test_dataframe_check_empty() {
        let df = DataFrame::new();
        assert_eq!(df.empty(), true);
    }

    #[test]
    fn test_test_dataframe_check_empty2() {
        let sql_datas = vec![("$A".to_string(), "voila".to_string()),
                         ("$A".to_string(), "element".to_string()),
                         ("$B".to_string(), "truc".to_string()),
                         ("$B".to_string(), "hey".to_string()),
                         ("$C".to_string(), "machin".to_string()),
                         ("$C".to_string(), "chose".to_string())];
        let df: Result<DataFrame, String> = sql_datas.try_into();
        assert_eq!(df.unwrap().empty(), false);
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

    #[test]
    fn test_dataframe_nb_rows() {
        let sql_datas = vec![("$A".to_string(), "voila".to_string()),
                         ("$A".to_string(), "element".to_string()),
                         ("$B".to_string(), "truc".to_string()),
                         ("$B".to_string(), "hey".to_string()),
                         ("$C".to_string(), "machin".to_string()),
                         ("$C".to_string(), "chose".to_string())];
        let df: DataFrame = sql_datas.try_into().unwrap();
        assert_eq!(df.nb_rows(), 2);
    }

    #[test]
    fn test_dataframe_nb_columns() {
        let sql_datas = vec![("$A".to_string(), "voila".to_string()),
                         ("$A".to_string(), "element".to_string()),
                         ("$B".to_string(), "truc".to_string()),
                         ("$B".to_string(), "hey".to_string()),
                         ("$C".to_string(), "machin".to_string()),
                         ("$C".to_string(), "chose".to_string())];
        let df: DataFrame = sql_datas.try_into().unwrap();
        assert_eq!(df.nb_columns(), 3);
    }

    #[test]
    fn test_dataframe_nb_rows2() {
        let sql_datas = vec![("$A".to_string(), "voila".to_string()),
                         ("$B".to_string(), "hey".to_string()),
                         ("$C".to_string(), "chose".to_string())];
        let df: DataFrame = sql_datas.try_into().unwrap();
        assert_eq!(df.nb_rows(), 1);
    }

}
