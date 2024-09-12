use std::fmt;
use regex::Regex;
use itertools::Itertools;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd(String);

impl Cmd {
    pub fn new(s: &str) -> Cmd {
        Cmd(String::from(s))
    }

    pub fn extract_variables(&self) -> Vec<String> {
        let re = Regex::new(r"\$(?P<variable>\w)").unwrap();
        re.captures_iter(&self.to_string())
            .map(|x| x["variable"].to_owned())
            .unique()
            .collect()
    }
}


impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cmd({})", self.0)
    }
}


impl Deref for Cmd {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Cmd {
   fn from(val: String) -> Self {
        Cmd(val)
   } 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variable() {
        assert_eq!(
            Cmd::new("add $C ami julie").extract_variables(),
            vec!["C".to_string()]);
    }

}

