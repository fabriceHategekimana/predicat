use core::fmt::Debug;
use std::collections::HashMap;
use parser::var::Var;


pub trait Context: Debug {
    type FellowContext;
    type DataError;
    fn get_variables(&self) -> Vec<Var>; // get column's names
    fn get_values(&self, key: &str) -> Result<Vec<String>, Self::DataError>; // get column's values
    fn get_values2(&self, columns: &[&str]) -> Option<Vec<Vec<String>>>;
    fn get_table(&self) -> HashMap<String, Vec<String>>; // get the whole table
    fn add_column(&mut self, name: &str, elements: &[&str]) -> Self;
    fn is_in_context(&self, key: String) -> bool;
    fn dataframe_len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn is_not_empty(&self) -> bool;
    fn new() -> Self;
    fn join(&self, c2: Self::FellowContext) -> Self::FellowContext;
}
