use polars::frame::DataFrame;
use crate::base_context::Context;
use polars::series::Series;
use polars::prelude::NamedFrom;

// TODO: implement context for DataFrame
impl Context for DataFrame {
    fn get_variables(&self) -> Vec<String>{
        vec!["1".to_string()]
    }
    fn get_values(&self, key: &str) -> Option<Vec<String>>{
        todo!();
    }

    fn add_column(&mut self, name: &str, elements: Vec<String>) -> &mut Self {
        let s = Series::new(name, elements);
        self.with_column(s);
        self
    }

    fn is_in_context(&self, key: String) -> bool{
        true
    }

    fn len(&self) -> usize{
        todo!();
    }
}


#[cfg(test)]
mod tests {
    use super::DataFrame;
    use crate::base_context::Context;

    #[test]
    fn test_context_get_variable(){
        let mut context = DataFrame::default();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(context.get_variables(), vec!["name"]);
    }

    #[test]
    fn test_is_in_context() {
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
    fn test_context_get_value(){
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
    fn test_simple_context_len() {
        let mut context = DataFrame::default();
        context = context.add_column("name", vec!["Vestin".to_string(), "Rédempta".to_string(), "Fabrice".to_string()]);
        assert_eq!(context.len(), 3);
    }

}
