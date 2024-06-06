use core::fmt::Debug;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Var(pub String);

impl Var {
    pub fn new(s: &str) -> Option<Self> {
        if s[0..1] == *"$" {
            Some(Var(s.to_string()))
        } else {
            None
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Var('{}')", self.0)
    }
}

pub trait Context: Debug {
    type FellowContext;
    fn get_variables(&self) -> Vec<Var>; // get column's names
    fn get_values(&self, key: &str) -> Option<Vec<String>>; // get column's values
    fn get_values2(&self, columns: &[&str]) -> Option<Vec<Vec<String>>>;
    fn get_table(&self) -> Vec<(String, String)>; // get the whole table
    fn add_column(&mut self, name: &str, elements: &[&str]) -> Self;
    fn is_in_context(&self, key: String) -> bool;
    fn dataframe_len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn is_not_empty(&self) -> bool;
    fn new() -> Self;
    fn join(&self, c2: Self::FellowContext) -> Self::FellowContext;
}
