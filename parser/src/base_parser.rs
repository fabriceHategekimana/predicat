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

use std::fmt;

use nom::bytes::complete::take_while;
use nom::character::complete::multispace0;
use nom::character::complete::one_of;
use nom::character::complete::none_of;
use crate::var::Var;

pub use crate::Triplet;

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

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
    Modifier(CommandType, Vec<Triplet>),
    Infer(String, String),
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
            PredicatAST::Modifier(CommandType::Add, v) => format!("add {}", v.iter().cloned().map(String::from).fold("".to_string(), |acc, x| format!("{} and {}", acc, x))),
            PredicatAST::Modifier(CommandType::Delete, v) => format!("add {}", v.iter().cloned().map(String::from).fold("".to_string(), |acc, x| format!("{} and {}", acc, x))),
            _ => "".to_string()
        }
    }    
}

#[derive(Debug, Clone)]
pub struct Modifier(pub CommandType, pub Vec<Triplet>);

impl Modifier {
    pub fn len(&self) -> usize {
        self.1.len()
    }
}

impl From<PredicatAST> for Modifier {
   fn from(val: PredicatAST) -> Self {
      match val {
          PredicatAST::Modifier(CommandType::Add, v) => Modifier(CommandType::Add, v.clone()),
          PredicatAST::Modifier(CommandType::Delete, v) => Modifier(CommandType::Delete, v.clone()),
          _ => Modifier(CommandType::Get, vec![])
      } 
   } 
}

#[derive(PartialEq, Debug, Clone)]
pub struct Comp(pub String);

impl Comp {
    pub fn get_content(&self) -> (String, String, String) {
        let val = self.0.split(" ").collect::<Vec<&str>>();
        (val[0].to_string(), val[1].to_string(), val[2].to_string())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Element {
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
    NotVar(String),
    Get,
    Connector,
    Element(Element),
    NotElement(Element),
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

impl TryFrom<Language> for Triplet {
    type Error = &'static str;

    fn try_from(l: Language) -> Result<Self, Self::Error> {
        match l {
            Language::Tri(t) => Ok(t),
            _ => Err("This is not a triplet")
        }
    }
}


pub fn extract_triplet(tri: &Language) -> Option<Triplet> {
    match tri {
        Language::Tri(tri) => Some(tri.clone()),
        _ => None
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

pub fn parse_pure_variable(s: &str) -> IResult<&str,Language> {
    preceded(
        space1,
        parse_variable_or_star
        )(s)
}

fn parse_not_variable(s: &str) -> IResult<&str,Language> {
    let res = preceded(tag(" not"), parse_pure_variable)(s);
    match res {
        Ok((s, Language::Var(v))) => Ok((s, Language::NotVar(v))),
        Ok((s, _)) => Ok((s, Language::Empty)),
        Err(r) => Err(r)
    }
}

pub fn parse_variable(s: &str) -> IResult<&str,Language> {
    alt((parse_not_variable, parse_pure_variable))(s)
}

fn alpha_num_underscore(s: &str) -> IResult<&str, String> {
    let res = many1(one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_"))(s);
    match res {
        Ok((s, v)) => Ok((s, v.iter().collect())),
        Err(e) => Err(e)
    }
}

fn parse_term(input: &str) -> IResult<&str, Element> {
    let res = alpha_num_underscore(input);
    match res {
        Ok((s, t)) => Ok((s, Element::Term(t.to_string()))),
        Err(e) => Err(e)
    }
}

fn string_content(s: &str) -> IResult<&str, String> {
    let res = many1(none_of("'"))(s);
    match res {
        Ok((s, v)) => Ok((s, v.iter().collect())),
        Err(r) => Err(r)
    }
}

fn parse_string(s: &str) -> IResult<&str, Element> {
    let res = delimited(tag("'"), string_content, tag("'"))(s);
    match res {
        Ok((t, s)) => Ok((t, Element::String(s))),
        Err(e) => Err(e)
    }
}


fn parse_pure_element(s: &str) -> IResult<&str,Language> {
    let res = preceded(
                multispace0,
                alt((parse_term,
                     parse_string)))(s);
    match res {
        Ok((t, e)) => Ok((t, Language::Element(e))),
        Err(e) => Err(e)
    }
}

fn parse_not_element(s: &str) -> IResult<&str,Language> {
    let res = preceded(tag(" not"), parse_pure_element)(s);
    match res {
        Ok((s, Language::Element(e))) => Ok((s, Language::NotElement(e))),
        Ok((s, _)) => Ok((s, Language::Empty)),
        Err(r) => Err(r)
    }
}

fn parse_element(s: &str) -> IResult<&str,Language> {
    alt((parse_not_element, parse_pure_element))(s)
}

pub fn parse_triplet(s: &str) -> IResult<&str,Language> {
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
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::Element(s3)))) 
            => Ok((t, Language::Tri(Triplet::Teee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::Tvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::Teve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::Teev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::Tvve(s1,s2,s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::Tvev(s1,s2.into(),s3)))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::Tevv(s1.into(),s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::Tvvv(s1,s2,s3)))),
        Err(e) 
			=> Err(e),
        // not first position
        Ok((t, (Language::NotElement(s1),Language::Element(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::NotVar(s1),Language::Element(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::NotElement(s1),Language::Var(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::NotElement(s1),Language::Element(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::NotVar(s1),Language::Var(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvve(s1,s2,s3.into())))),
        Ok((t, (Language::NotVar(s1),Language::Element(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvev(s1,s2.into(),s3)))),
        Ok((t, (Language::NotElement(s1),Language::Var(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNevv(s1.into(),s2,s3)))),
        Ok((t, (Language::NotVar(s1),Language::Var(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvvv(s1,s2,s3)))),
        // not second position
        Ok((t, (Language::Element(s1),Language::NotElement(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::Var(s1),Language::NotElement(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::Element(s1),Language::NotVar(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::Element(s1),Language::NotElement(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::Var(s1),Language::NotVar(s2),Language::Element(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvve(s1,s2,s3.into())))),
        Ok((t, (Language::Var(s1),Language::NotElement(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvev(s1,s2.into(),s3)))),
        Ok((t, (Language::Element(s1),Language::NotVar(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNevv(s1.into(),s2,s3)))),
        Ok((t, (Language::Var(s1),Language::NotVar(s2),Language::Var(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvvv(s1,s2,s3)))),
        // not third position
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::NotElement(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::NotElement(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::NotElement(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::NotVar(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNeev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::NotElement(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvve(s1,s2,s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::NotVar(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvev(s1,s2.into(),s3)))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::NotVar(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNevv(s1.into(),s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::NotVar(s3)))) 
			=> Ok((t, Language::Tri(Triplet::TNvvv(s1,s2,s3)))),
        Ok((t, _)) => Ok((t, Language::Empty))
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
            parse_term("wow").unwrap().1,
            Element::Term("wow".to_string()));
        assert_eq!(
            parse_term("$A"),
            Err(nom::Err::Error(
                Error {
                    input: "$A",
                    code: ErrorKind::OneOf
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
