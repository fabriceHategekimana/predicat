use core::fmt::Debug;

pub trait Context: Debug {
    type FellowContext;
    fn new() -> Self;
    fn get_variables(&self) -> Vec<String>;
    fn get_values(&self, key: &str) -> Option<Vec<String>>;
    fn add_column(&mut self, name: &str, elements: Vec<String>) -> Self;
    fn is_in_context(&self, key: String) -> bool;
    fn len(&self) -> usize;
    fn join(&self, c2:Self::FellowContext) -> Self::FellowContext;
}
