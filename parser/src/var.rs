use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Var(pub String);

impl Var {
    pub fn new(s: &str) -> Self {
        if Self::contains_dollar(s) { Var(s.to_string()) } else { Var(format!("${}", s)) }
    }

    fn contains_dollar(s: &str) -> bool {
        &s[..1] == "$"
    }

    pub fn format(s: &str) -> String {
        if Self::contains_dollar(s) { s.to_string() } else { format!("${}", s) }
    }

    pub fn without_dollar(&self) -> String {
        self[1..].to_string()
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_creation(){
        assert_eq!(Var::new("a"), Var("$a".to_string()));        
    }

    #[test]
    fn test_var_format() {
        assert_eq!(Var::format("a"), "$a");
    }
}
