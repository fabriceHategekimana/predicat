use core::fmt::Debug;

pub trait Context: Debug {
    type FellowContext;
    fn new() -> Self;
    fn get_variables(&self) -> Vec<String>; // get column's names
    fn get_values(&self, key: &str) -> Option<Vec<String>>; // get column's values
    fn get_table(&self) -> Vec<(String, String)>; // get the whole table
    fn add_column(&mut self, name: &str, elements: &[&str]) -> Self;
    fn is_in_context(&self, key: String) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn is_not_empty(&self) -> bool;
    fn join(&self, c2: Self::FellowContext) -> Self::FellowContext;
}

