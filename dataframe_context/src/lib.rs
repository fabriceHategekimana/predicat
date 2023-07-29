use polars::frame::DataFrame;
use polars::series::Series;
use polars::prelude::NamedFrom;

pub trait Context {
    fn new() -> Self;
    fn get_variables(&self) -> Vec<String>;
    fn get_values(&self, key: &str) -> Option<Vec<String>>;
    fn add_column(&mut self, name: &str, elements: Vec<String>) -> Self;
    fn is_in_context(&self, key: String) -> bool;
    fn len(&self) -> usize;
}

impl Context for DataFrame {

    fn new() -> DataFrame {
        let res = DataFrame::default();
        res
    }

    fn get_variables(&self) -> Vec<String> {
        self.get_columns().iter().map(|x| x.name().to_string()).collect()
    }

    fn get_values(&self, key: &str) -> Option<Vec<String>>{
        match self.is_in_context(key.to_string()) {
            false => None, 
            true => Some(self[key].utf8()
                                  .ok()?
                                  .into_no_null_iter()
                                  .map(|x| x.to_string())
                                  .collect())
        }
    }

    fn add_column(&mut self, name: &str, elements: Vec<String>) -> Self {
        let columns = self.get_columns();
        let s = Series::new(name, elements);
        DataFrame::new(columns.iter()
                              .chain([&s])
                              .map(|x| x.clone())
                              .collect::<Vec<Series>>())
                              .unwrap()
    }

    fn is_in_context(&self, key: String) -> bool{
        self.get_variables().iter().any(|x| x == &key)
    }

    fn len(&self) -> usize{
        self.height()
    }
}


#[cfg(test)]
mod tests {
    use super::DataFrame;
    use super::Context;

    #[test]
    fn test_context_get_variable_dataframe(){
        let mut context = DataFrame::default();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(context.get_variables(), vec!["name"]);
    }

    #[test]
    fn test_is_in_context_dataframe() {
        let mut context = DataFrame::default();
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
    fn test_context_get_value_dataframe(){
        let mut context = DataFrame::default();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(
            context.get_values("name"),
            Some(vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]));
        assert_eq!(
            context.get_values("truc"),
            None);
    }

    #[test]
    fn test_simple_context_len_dataframe() {
        let mut context = DataFrame::default();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(context.len(), 3);
    }

}
