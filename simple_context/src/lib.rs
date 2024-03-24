use cli_table::{Style, Table};
use itertools::Itertools;
use base_context::Context;

type ColumnName = String;
type Value = String;

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct SimpleContext {
    pub tab: Vec<(ColumnName, Value)>, 
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
        self.log == vec![] as Vec<String>
    }


pub fn display(&self) {
    // TODO : display error if any and call revert back in the back
    match self.len() {
        x if x > 0 => {
        let variables = self.get_variables();
        let body = self.get_variables().iter()
            .map(|x| self.get_values(x).unwrap())
            .collect::<Vec<_>>();
        let table = (0..self.len()).map(|x| get_line(x, &body))
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

    pub fn get_tab(&self) -> Vec<(String, String)> {
        self.tab.clone()
    }
}

fn get_line(num: usize, body: &[Vec<String>]) -> Vec<String> {
    body.iter().map(|x| x[num].clone()).collect()
}

impl From<Vec<(String, String)>> for SimpleContext {
    fn from(v: Vec<(String, String)>) -> SimpleContext {
        SimpleContext { tab: v, cmds: vec![], log: vec![] }
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


impl Context for SimpleContext {

    type FellowContext = SimpleContext;

    fn new() -> SimpleContext {
        SimpleContext{
            tab: vec![],
            cmds: vec![],
            log: vec![]
        }
    }

    fn get_variables(&self) -> Vec<String>{
        self.tab.iter().map(|x| x.0.clone()).sorted().unique().collect()
    }

    fn get_values(&self, key: &str) -> Option<Vec<String>> {
        match self.is_in_context(key.to_string()) {
            true => Some(self.tab.iter().filter(|x| x.0.clone() == key).map(|x| x.1.clone()).collect::<Vec<String>>()),
            _ => None
        }
    }

    fn add_column(&mut self, name: &str, elements: &[&str]) -> SimpleContext {
        let tab = elements.iter()
                          .map(|x| (name.to_string(), x.to_string()))
                          .collect::<Vec<(String, String)>>();
        let new_tab = self.tab.iter().chain(tab.iter()).map(|x| x.clone()).collect::<Vec<_>>();
        SimpleContext::from(new_tab)
    }

    fn is_in_context(&self, key: String) -> bool {
        self.get_variables().iter().any(|x| &x[..] == key)
    }

    fn len(&self) -> usize {
        match self.tab.len() {
            0 => 0,
            _ => self.get_values(&self.tab[0].0).unwrap().len()
        }
    }

    fn join(&self, c2: SimpleContext) -> SimpleContext {
        let vec = self.get_tab()
                    .iter()
                    .chain(c2.get_tab().iter())
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();
        SimpleContext {
            tab: vec.clone(),
            ..c2
        }
    }
    
    fn is_empty(&self) -> bool {
        self.len() > 0
    }

    fn is_not_empty(&self) -> bool {
       !self.is_empty() 
    }

    fn get_aftercmds(&self) -> Vec<String> {
        self.cmds.clone()
    }

    fn get_table(&self) -> Vec<(String, String)> {
        self.tab.clone()
    }
}

