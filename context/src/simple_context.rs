use itertools::Itertools;
use crate::base_context::Context;


#[derive(Debug)]
struct SimpleContext {
    tab: Vec<(String, String)> 
}

//Constructor
impl SimpleContext {
    fn new() -> Self {
        SimpleContext{
            tab: vec![]
        }
    }
    fn from(entry: Vec<(String, String)>) -> SimpleContext {
        SimpleContext{
            tab: entry
        }
    }
}


impl Context for SimpleContext {
    fn get_variables(&self) -> Vec<String>{
        self.tab.iter().map(|x| x.0.clone()).sorted().unique().collect()
    }

    fn get_values(&self, key: &str) -> Option<Vec<String>> {
        match self.is_in_context(key.to_string()) {
            true => Some(self.tab.iter().filter(|x| x.0.clone() == key).map(|x| x.1.clone()).collect::<Vec<String>>()),
            _ => None
        }
    }

    fn add_column(&mut self, name: &str, elements: Vec<String>) -> &mut SimpleContext {
        let tab = elements.iter()
                          .map(|x| (name.to_string(), x.to_string()))
                          .collect::<Vec<(String, String)>>();
        &mut SimpleContext::from(tab)
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
}

#[cfg(test)]
mod tests {
    use super::SimpleContext;
    use crate::base_context::Context;

    #[test]
    fn test_context_get_variable(){
        let mut context = &mut SimpleContext::new();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(context.get_variables(), vec!["name"]);
    }

    #[test]
    fn test_is_in_context() {
        let mut context = &mut SimpleContext::new();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
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
        let mut context = &mut SimpleContext::new();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(
            context.get_values("name"),
            Some(vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]));
        assert_eq!(
            context.get_values("truc"),
            None);
    }

    #[test]
    fn test_simple_context_len() {
        let mut context = SimpleContext::new();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(context.len(), 3);
    }
}

