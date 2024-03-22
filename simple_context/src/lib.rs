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

#[cfg(test)]
mod tests {
    use super::SimpleContext;
    use base_context::Context;

    #[test]
    fn test_context_get_variable(){
        let mut context = SimpleContext::new();
        context = context.add_column("name", &["Vestin", "Rédempta", "Fabrice"]);
        context = context.add_column("age", &["28", "23", "28"]);
        assert_eq!(context.get_variables(), vec!["age", "name"]);
    }

    #[test]
    fn test_len(){
        let mut context = SimpleContext::new();
        context = context.add_column("name", &["Vestin", "Rédempta", "Fabrice"]);
        context = context.add_column("age", &["28", "23", "28"]);
        assert_eq!(context.len(), 3);
    }

    #[test]
    fn test_is_in_context() {
        let mut context = SimpleContext::new();
        context = context.add_column("name", &["Vestin", "Rédempta", "Fabrice"]);
        assert_eq!(
            context.is_in_context("name".to_string()),
            true
            );
        assert_eq!(
            context.is_in_context("truc".to_string()),
            false
            );
    }

    #[test]
    fn test_context_get_value(){
        let mut context = SimpleContext::new();
        context = context.add_column( "name", &["Vestin", "Rédempta", "Fabrice"]);
        context = context.add_column("age", &["28", "23", "28"]);
        assert_eq!(
            context.get_values("name"),
            Some(vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]));
        assert_eq!(
            context.get_values("truc"),
            None);
        assert_eq!(
            context.get_values("age"),
            Some(vec!["28".to_string(), "23".to_string(), "28".to_string()]));
    }

    #[test]
    fn test_simple_context_len() {
        let mut context = SimpleContext::new();
        context = context.add_column("name", &["Vestin", "Rédempta", "Fabrice"]);
        assert_eq!(context.len(), 3);
    }


    #[test]
    fn test_context_get_variable2(){
        let context = SimpleContext::from(
                        &vec![("A".to_string(), "emy".to_string()), ("B".to_string(), "ami".to_string()), ("C".to_string(), "alice".to_string())]);
        assert_eq!(context.get_variables(), vec!["A", "B", "C"]);
    }

    #[test]
    fn test_context_get_value2(){
        let context = SimpleContext::from(
                        &vec![("A".to_string(), "emy".to_string()), ("B".to_string(), "ami".to_string()), ("C".to_string(), "alice".to_string())]);
        assert_eq!(
            context.get_values("A"),
            Some(vec!["emy".to_string()]));
        assert_eq!(
            context.get_values("C"),
            Some(vec!["alice".to_string()]));
    }

    #[test]
    fn test_join() {
        let context1 = SimpleContext::from(&[("A".to_string(), "pierre".to_string())]);
        let context2 = SimpleContext::from(&[("B".to_string(), "jean".to_string())]);
        let joined_context = context1.join(context2);
        let mut context = SimpleContext::new();
        context = context.add_column("A", &["pierre"]);
        context = context.add_column("B", &["jean"]);
        assert_eq!(
            joined_context,
            context
                  );
    }

}
