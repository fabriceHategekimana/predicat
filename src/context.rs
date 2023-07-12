use std::collections::HashMap;

trait Context {
    fn get_variables(&self) -> Vec<&str>;
    fn get_values(&self, key: &str) -> Option<&[&str]>;
    fn add_column(&mut self, name: &str, elements: &[&str]);
}

struct SimpleContext<'a> {
    tab: HashMap<&'a str, &'a [&'a str]>
}

impl<'a> SimpleContext<'a> {
    fn new() -> SimpleContext<'a> {
        SimpleContext{
            tab: HashMap::new()
        }
    }
}

impl Context for SimpleContext<'_> {
    fn get_variables(&self) -> Vec<&str>{
        self.tab.keys().map(|x| *x).collect()
    }

    fn get_values(&self, key: &str) -> Option<&[&str]> {
        self.tab.get(key).cloned()
    }

    fn add_column<'a>(&'a mut self, name: &str, elements: &[&'a str]) {
        self.tab.insert(name, elements);
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleContext;
    use crate::context::Context;

    #[test]
    fn test_context(){
        let mut context = SimpleContext::new();
        context.add_column("A", vec!["1".to_string(), "2".to_string(), "3".to_string()]);
        assert_eq!(context.get_values("A"), Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]));
    }
}
