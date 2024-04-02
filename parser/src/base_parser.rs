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

use nom::bytes::complete::take_while;
use nom::character::complete::multispace0;

use base_context::simple_context::SimpleContext;
pub use Triplet::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Action {
    Block,
    Infer
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CommandType {
    Add,
    Delete,
    Get
}

impl CommandType {
    pub fn get_string(&self) -> String {
        match self {
            CommandType::Add => "add".to_string(),
            CommandType::Delete => "delete".to_string(),
            CommandType::Get => "get".to_string(),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Command {
    Str(String, String),
    Predicat(String, Box<PredicatAST>)
}

#[derive(PartialEq, Debug, Clone)]
pub enum PredicatAST {
    Query(
        (Vec<Var>,
         Vec<Triplet>,
         Vec<Comp>)),
    AddModifier(Vec<Triplet>),
    DeleteModifier(Vec<Triplet>),
    Infer((CommandType, Vec<Triplet>), String),
    // TODO: add Block and Assert rules
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

impl From<PredicatAST> for String {
    fn from(p: PredicatAST) -> String {
        match p {
            PredicatAST::AddModifier(v) => format!("add {}", v.iter().cloned().map(String::from).fold("".to_string(), |acc, x| format!("{} and {}", acc, x))),
            PredicatAST::DeleteModifier(v) => format!("add {}", v.iter().cloned().map(String::from).fold("".to_string(), |acc, x| format!("{} and {}", acc, x))),
            _ => "".to_string()
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

#[derive(PartialEq, Debug, Clone)]
enum Element {
    Term(String),
    String(String)
}

impl From<Element> for String {
    fn from(e: Element) -> String {
        match e {
            Element::Term(w) => w,
            Element::String(s) => s
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Language {
    Var(String),
    Get,
    Connector,
    Element(Element),
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
    Teee(String, String, String),
    Tvee(String, String, String),
    Teve(String, String, String),
    Teev(String, String, String),
    Tvve(String, String, String),
    Tvev(String, String, String),
    Tevv(String, String, String),
    Tvvv(String, String, String),
    Empty
}

impl TryFrom<Language> for Triplet {
    type Error = &'static str;

    fn try_from(l: Language) -> Result<Self, Self::Error> {
        match l {
            Language::Tri(t) => Ok(t),
            _ => Err("This is not a triplet")
        }
    }
}

impl From<Triplet> for (String, String, String) {
    fn from(t: Triplet) -> (String, String, String) {
        match t {
           Triplet::Teee(a, b, c) => (a, b, c),
           Triplet::Tvev(a, b, c) => (a, b, c),
           Triplet::Tevv(a, b, c) => (a, b, c),
           Triplet::Teev(a, b, c) => (a, b, c),
           Triplet::Teve(a, b, c) => (a, b, c),
           Triplet::Tvve(a, b, c) => (a, b, c),
           Triplet::Tvee(a, b, c) => (a, b, c),
           Triplet::Tvvv(a, b, c) => (a, b, c),
           Triplet::Empty => ("".to_string(), "".to_string(), "".to_string())
        }
    }
}

impl From<Triplet> for String {
    fn from(t: Triplet) -> String {
        let tri : (String, String, String) = t.into();
        format!("{} {} {}", tri.0, tri.1, tri.2)
    }
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
    pub fn to_tuple(&self) -> (String, String, String) {
        match self {
            Teee(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvee(a,b,c) => format_tuple_of_three((a,b,c)),
            Teve(a,b,c) => format_tuple_of_three((a,b,c)),
            Teev(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvve(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvev(a,b,c) => format_tuple_of_three((a,b,c)),
            Tevv(a,b,c) => format_tuple_of_three((a,b,c)),
            Tvvv(a,b,c) => format_tuple_of_three((a,b,c)),
            _ => todo!()
        }
    }
    pub fn to_tuple_with_variable(&self) -> (String, String, String) {
        match self {
            Teee(a,b,c) => (a.to_string(),b.to_string(),c.to_string()),
            Tvee(a,b,c) => (to_var(&a),b.to_string(),c.to_string()),
            Teve(a,b,c) => (a.to_string(),to_var(&b),c.to_string()),
            Teev(a,b,c) => (a.to_string(),b.to_string(), to_var(&c)),
            Tvve(a,b,c) => (to_var(&a),to_var(&b),c.to_string()),
            Tvev(a,b,c) => (to_var(&a),b.to_string(),to_var(&c)),
            Tevv(a,b,c) => (a.to_string(),to_var(&b),to_var(&c)),
            Tvvv(a,b,c) => (to_var(&a),to_var(&b),to_var(&c)),
            _ => todo!()
        }
    }

    pub fn display(&self) -> String {
        match self {
            Teee(a,b,c) => format!("{},{},{}", a.to_string(),b.to_string(),c.to_string()),
            Tvee(a,b,c) => format!("{},{},{}", to_var(&a),b.to_string(),c.to_string()),
            Teve(a,b,c) => format!("{},{},{}", a.to_string(),to_var(&b),c.to_string()),
            Teev(a,b,c) => format!("{},{},{}", a.to_string(),b.to_string(), to_var(&c)),
            Tvve(a,b,c) => format!("{},{},{}", to_var(&a),to_var(&b),c.to_string()),
            Tvev(a,b,c) => format!("{},{},{}", to_var(&a),b.to_string(),to_var(&c)),
            Tevv(a,b,c) => format!("{},{},{}", a.to_string(),to_var(&b),to_var(&c)),
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

fn parse_term(input: &str) -> IResult<&str, Element> {
    let res = preceded(
            multispace0,
            take_while(|c: char| c.is_alphanumeric() || c == '_')
        )(input);
        //.map(|(next_input, output)| (next_input, output.0.trim()))
    match res {
        Ok((s, t)) => Ok((s, Element::Term(t.to_string()))),
        Err(e) => Err(e)
    }
}

fn parse_string(s: &str) -> IResult<&str, Element> {
    todo!();
}

fn parse_element(s: &str) -> IResult<&str,Language> {
    let res = alt((
                parse_term,
                parse_string))(s);
    match res {
        Ok((t, e)) => Ok((t, Language::Element(e))),
        Err(e) => Err(e)
    }
}

fn parse_triplet(s: &str) -> IResult<&str,Language> {
    let res = alt((
            tuple((parse_element, parse_element, parse_element)),
            tuple((parse_variable, parse_element, parse_element)),
            tuple((parse_element, parse_variable, parse_element)),
            tuple((parse_element, parse_element, parse_variable)),
            tuple((parse_variable, parse_variable, parse_element)),
            tuple((parse_variable, parse_element, parse_variable)),
            tuple((parse_element, parse_variable, parse_variable)),
            tuple((parse_variable, parse_variable, parse_variable))
            ))(s);
    match res {
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Teee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Tvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Teve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Teev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Tvve(s1,s2,s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Tvev(s1,s2.into(),s3)))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Tevv(s1.into(),s2,s3)))),
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
        parse_term,
        parse_bar,
        parse_triplet,
        parse_triplet_and,
        Language,
        Triplet::*,
        Error
    };
    use crate::base_parser::Element;

    #[test]
    fn test_term() {
        assert_eq!(
            parse_term(" wow").unwrap().1,
            Element::Term("wow".to_string()));
        assert_eq!(
            parse_term(" $A"),
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
            Language::Tri(Teee("un".to_string(), "deux".to_string(), "trois".to_string())));
        assert_eq!(
            parse_triplet(" un deux $A").unwrap().1,
            Language::Tri(Teev("un".to_string(), "deux".to_string(), "A".to_string())));
        assert_eq!(
            parse_triplet(" $A deux trois").unwrap().1,
            Language::Tri(Tvee("A".to_string(), "deux".to_string(), "trois".to_string())));
    }

    #[test]
    fn test_triplet_and() {
        assert_eq!(
            parse_triplet_and(" B ami C AND A ami C").unwrap().1,
            Language::Tri(Teee("B".to_string(),"ami".to_string(),"C".to_string()))
        );
    }

    #[test]
    fn test_parse_bar() {
        assert_eq!(
            parse_bar(" | ").unwrap().1,
            " | ");
    }

}
