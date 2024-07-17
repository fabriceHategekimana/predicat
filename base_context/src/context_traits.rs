use core::fmt::Debug;
use std::ops::Deref;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Var(pub String);

impl Var {
    pub fn new(s: &str) -> Self {
        if s[0..1] == *"$" {
            Var(s.to_string())
        } else {
            Var(format!("${}", s))
        }
    }

    pub fn without_dollar(&self) -> String {
        self[1..].to_string()
    }

    pub fn format(s: &str) -> String {
        Self::new(s).0.to_string()
    }
}

impl Deref for Var {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Var('{}')", self.0)
    }
}

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
