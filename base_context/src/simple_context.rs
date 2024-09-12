use cli_table::{Style, Table};
use crate::context_traits::Context;
use std::collections::HashMap;
use parser::cmd::Cmd;
use parser::var::Var;
use crate::dataframe::DataFrame;
use crate::dataframe::DataFrameError;


pub trait Adder {
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


#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct SimpleContext {
    pub tab: DataFrame,
    pub cmds: Vec<Cmd>,
    pub log: Vec<String>
}

impl SimpleContext {
    pub fn join_contexts(ctx1: SimpleContext, ctx2: SimpleContext) -> SimpleContext {
        ctx1.join(ctx2)
    }

    pub fn has_commands(&self) -> bool {
        self.cmds != vec![] as Vec<Cmd>
    }

    pub fn has_error(&self) -> bool {
        self.log != vec![] as Vec<String>
    }

    pub fn len(&self) -> usize {
        self.tab.len()
    }

    pub fn show(&self) {
        let tab = self.get_tab();
        if !tab.empty() {
                let variables = tab.get_variables().iter()
                    .map(Var::without_dollar).collect::<Vec<String>>();
                let body = tab.get_variables().iter()
                    .map(|x| tab.get_values(x).unwrap_or(vec![]))
                    .collect::<Vec<_>>();
                let table = (0..tab.len()).map(|x| get_line(x, &body))
                    .table()
                    .title(variables)
                    .bold(true)
                    .display()
                    .unwrap();
                    println!("{}", table);
            } else {
                println!("[]")
        }
    }


    pub fn get_tab(&self) -> DataFrame {
        self.tab.clone()
    }

    pub fn duplicate_command(&self, command: &Cmd) -> Vec<String> {
        (0..self.dataframe_len()).into_iter().map(|_x| command.to_string()).collect()
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
        self.tab.get_cells()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    //#[test]
    //fn test_duplicate_commande() {
        //let mut context = SimpleContext::new();
        //context = context.add_column("$A", &["pierre", "anne", "murielle"]);
        //assert_eq!(
            //context.len(),
            //3);
        //assert_eq!(
            //context.duplicate_command(&Cmd::new("add $A ami julie")),
            //vec!["add $A ami julie".to_string(), "add $A ami julie".to_string(), "add $A ami julie".to_string()]
                  //);
    //}


}

