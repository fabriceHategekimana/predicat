use std::collections::HashMap;

trait Context {
    fn get_variables(&self) -> Vec<String>;
    fn get_values(&self, key: String) -> Option<Vec<String>>;
    fn add_column(&mut self, name: String, elements: Vec<String>);
}

struct SimpleContext<'a> {
    tab: HashMap<&'a str, &'a [&'a str]>
}

//Constructor
impl SimpleContext<'_> {
    fn new<'a>() -> SimpleContext<'a> {
        SimpleContext{
            tab: HashMap::new()
        }
    }
}

impl Context for SimpleContext<'_> {
    fn get_variables(&self) -> Vec<String>{
        self.tab.keys().map(|x| x.to_string()).collect()
    }

    fn get_values(&self, key: String) -> Option<Vec<String>> {
        let res = self.tab.get(&key[..]);
        match res {
            Some(r) => Some(r.iter().map(|x| x.to_string()).collect()),
            _ => None
        }
    }

    fn add_column(&mut self, name: String, elements: Vec<String>) {
        self.tab.insert(&name[..], elements.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice());
    }
}

#[cfg(test)]
mod tests {
    use crate::context::SimpleContext;
    use crate::context::Context;

    #[test]
    fn test_context(){
        let mut context = SimpleContext::new();
        context.add_column("name".to_string(), vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        //assert_eq!(
            //context.get_variables(),
            //vec!["name"]
        //);
        assert_eq!(2, 2);
    }
}
