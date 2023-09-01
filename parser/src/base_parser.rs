pub use nom::{
    bytes::complete::{tag, is_not},
    character::complete::{char, alphanumeric1, space1, digit1},
    sequence::{preceded, tuple, delimited, terminated},
    branch::alt,
    combinator::recognize,
    multi::many1,
    error::{Error,
            ErrorKind},
    IResult
};

use simple_context::SimpleContext;

pub use Triplet::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Event {
    Before,
    After
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ModifierType {
    Add,
    Delete
}

#[derive(PartialEq, Debug, Clone)]
pub enum PredicatAST {
    Query(
        (Vec<Var>,
         Vec<Triplet>,
         Vec<Comp>)),
    AddModifier(Vec<Triplet>),
    DeleteModifier(Vec<Triplet>),
    // rule [event] [trigger] [action] 
    Rule(Event, (ModifierType, Triplet), String, Box<PredicatAST>),
    Empty,
    Debug(String)
}

impl PredicatAST {

    pub fn is_query(&self) -> bool {
        match self {
            PredicatAST::Query(q) => true,
            _ => false
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Var(pub String);

#[derive(PartialEq, Debug, Clone)]
pub struct Comp(pub String);

impl Comp {
    pub fn get_content(&self) -> (String, String, String) {
        let val = self.0.split(" ").collect::<Vec<&str>>();
        (val[0].to_string(), val[1].to_string(), val[2].to_string())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Language {
    Var(String),
    Get,
    Connector,
    Word(String),
    Tri(Triplet),
    Comp(String),
    Empty
}

impl Language {
    pub fn get_var(&self) -> Option<Var> {
        match self {
            Language::Var(s) => Some(Var(s.to_string())),
            _ => None
        }
    }
    pub fn get_comp(&self) -> Option<Comp> {
        match self {
            Language::Comp(s) => Some(Comp(s.to_string())),
            _ => None
        }
    }
}


pub fn extract_triplet(tri: &Language) -> Option<Triplet> {
    match tri {
        Language::Tri(tri) => Some(tri.clone()),
        _ => None
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Triplet {
    Twww(String, String, String),
    Tvww(String, String, String),
    Twvw(String, String, String),
    Twwv(String, String, String),
    Tvvw(String, String, String),
    Tvwv(String, String, String),
    Twvv(String, String, String),
    Tvvv(String, String, String),
    Empty
}


fn to_var(s: &str) -> String {
    format!("${}", s)
}

fn format_tuple_of_three(t: (&String, &String, &String)) -> (String, String, String) {
    match t {
        (a, b, c) => (a.to_owned(), b.to_owned(), c.to_owned()) 
    } 
}

impl Triplet {
    fn to_tuple(&self) -> (String, String, String) {
        match self {
            Twww(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvww(a,b,c) => format_tuple_of_three((a,b,c)),
            Twvw(a,b,c) => format_tuple_of_three((a,b,c)),
            Twwv(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvvw(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvwv(a,b,c) => format_tuple_of_three((a,b,c)),
            Twvv(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvvv(a,b,c) => format_tuple_of_three((a,b,c)),
            _ => todo!()
        }
    }
    pub fn to_tuple_with_variable(&self) -> (String, String, String) {
        match self {
            Twww(a,b,c) => (a.to_string(),b.to_string(),c.to_string()),
            Tvww(a,b,c) => (to_var(&a),b.to_string(),c.to_string()),
            Twvw(a,b,c) => (a.to_string(),to_var(&b),c.to_string()),
            Twwv(a,b,c) => (a.to_string(),b.to_string(), to_var(&c)),
            Tvvw(a,b,c) => (to_var(&a),to_var(&b),c.to_string()),
            Tvwv(a,b,c) => (to_var(&a),b.to_string(),to_var(&c)),
            Twvv(a,b,c) => (a.to_string(),to_var(&b),to_var(&c)),
            Tvvv(a,b,c) => (to_var(&a),to_var(&b),to_var(&c)),
            _ => todo!()
        }
    }

    pub fn display(&self) -> String {
        match self {
            Twww(a,b,c) => format!("{},{},{}", a.to_string(),b.to_string(),c.to_string()),
            Tvww(a,b,c) => format!("{},{},{}", to_var(&a),b.to_string(),c.to_string()),
            Twvw(a,b,c) => format!("{},{},{}", a.to_string(),to_var(&b),c.to_string()),
            Twwv(a,b,c) => format!("{},{},{}", a.to_string(),b.to_string(), to_var(&c)),
            Tvvw(a,b,c) => format!("{},{},{}", to_var(&a),to_var(&b),c.to_string()),
            Tvwv(a,b,c) => format!("{},{},{}", to_var(&a),b.to_string(),to_var(&c)),
            Twvv(a,b,c) => format!("{},{},{}", a.to_string(),to_var(&b),to_var(&c)),
            Tvvv(a,b,c) => format!("{},{},{}", to_var(&a),to_var(&b),to_var(&c)),
            _ => todo!()
        }
    }
}

pub fn parse_bar(s: &str) -> IResult<&str, &str> {
    alt((
            tag(" | "),
            tag("| "),
            tag(" |"),
            tag("|")
        ))(s)
}

fn parse_variable_or_star(s: &str) -> IResult<&str, Language> {
    let res = alt((
            preceded(char('$'), alphanumeric1),
            tag("*")
            ))(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Var(s.to_string()))),
        Err(e) => Err(e)
    }
}

pub fn parse_variable(s: &str) -> IResult<&str,Language> {
    preceded(
        space1,
        parse_variable_or_star
        )(s)
}

fn parse_word(s: &str) -> IResult<&str,Language> {
    let res = preceded(space1, alphanumeric1)(s);
    match res {
        Ok((t, s)) => Ok((t, Language::Word(s.to_string()))),
        Err(e) => Err(e)
    }
}

fn parse_triplet(s: &str) -> IResult<&str,Language> {
    let res = alt((
            tuple((parse_word, parse_word, parse_word)),
            tuple((parse_variable, parse_word, parse_word)),
            tuple((parse_word, parse_variable, parse_word)),
            tuple((parse_word, parse_word, parse_variable)),
            tuple((parse_variable, parse_variable, parse_word)),
            tuple((parse_variable, parse_word, parse_variable)),
            tuple((parse_word, parse_variable, parse_variable)),
            tuple((parse_variable, parse_variable, parse_variable))
            ))(s);
    match res {
        Ok((t, (Language::Word(s1),Language::Word(s2),Language::Word(s3)))) => Ok((t, Language::Tri(Twww(s1,s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Word(s2),Language::Word(s3)))) => Ok((t, Language::Tri(Tvww(s1,s2,s3)))),
        Ok((t, (Language::Word(s1),Language::Var(s2),Language::Word(s3)))) => Ok((t, Language::Tri(Twvw(s1,s2,s3)))),
        Ok((t, (Language::Word(s1),Language::Word(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Twwv(s1,s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::Word(s3)))) => Ok((t, Language::Tri(Tvvw(s1,s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Word(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Tvwv(s1,s2,s3)))),
        Ok((t, (Language::Word(s1),Language::Var(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Twvv(s1,s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Tvvv(s1,s2,s3)))),
        Err(e) => Err(e),
        _ => Ok(("", Language::Empty))
    }
}

pub fn parse_triplet_and(s: &str) -> IResult<&str,Language> {
    alt((
        terminated(parse_triplet, tag(" and")),
        parse_triplet))(s)
}

#[cfg(test)]
mod tests {
    use super::{
        ErrorKind,
        parse_word,
        parse_bar,
        parse_triplet,
        parse_triplet_and,
        Language,
        Triplet::*,
        Error
    };

    #[test]
    fn test_word() {
        assert_eq!(
            parse_word(" wow").unwrap().1,
            Language::Word("wow".to_string()));
        assert_eq!(
            parse_word(" $A"),
            Err(nom::Err::Error(
                Error {
                    input: "$A",
                    code: ErrorKind::AlphaNumeric
                }
            )));
    }

    #[test]
    fn test_triplet() {
        assert_eq!(
            parse_triplet(" un deux trois").unwrap().1,
            Language::Tri(Twww("un".to_string(), "deux".to_string(), "trois".to_string())));
        assert_eq!(
            parse_triplet(" un deux $A").unwrap().1,
            Language::Tri(Twwv("un".to_string(), "deux".to_string(), "A".to_string())));
        assert_eq!(
            parse_triplet(" $A deux trois").unwrap().1,
            Language::Tri(Tvww("A".to_string(), "deux".to_string(), "trois".to_string())));
    }

    #[test]
    fn test_triplet_and() {
        assert_eq!(
            parse_triplet_and(" B ami C AND A ami C").unwrap().1,
            Language::Tri(Twww("B".to_string(),"ami".to_string(),"C".to_string()))
        );
    }

    #[test]
    fn test_parse_bar() {
        assert_eq!(
            parse_bar(" | ").unwrap().1,
            " | ");
    }

}
