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
use nom::character::complete::one_of;
use nom::character::complete::none_of;

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
    TNeee(String, String, String),
    TNvee(String, String, String),
    TNeve(String, String, String),
    TNeev(String, String, String),
    TNvve(String, String, String),
    TNvev(String, String, String),
    TNevv(String, String, String),
    TNvvv(String, String, String),
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
           Triplet::TNeee(a, b, c) => (format!("not_{}", a), b, c),
           Triplet::TNvev(a, b, c) => (format!("not_{}", a), b, c),
           Triplet::TNevv(a, b, c) => (format!("not_{}", a), b, c),
           Triplet::TNeev(a, b, c) => (format!("not_{}", a), b, c),
           Triplet::TNeve(a, b, c) => (format!("not_{}", a), b, c),
           Triplet::TNvve(a, b, c) => (format!("not_{}", a), b, c),
           Triplet::TNvee(a, b, c) => (format!("not_{}", a), b, c),
           Triplet::TNvvv(a, b, c) => (format!("not_{}", a), b, c),
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
    pub fn invert(self) -> Triplet {
        match self {
            Teee(a,b,c) => Teee(a,b,c),
            Tvee(a,b,c) => Tvee(a,b,c),
            Teve(a,b,c) => Teve(a,b,c),
            Teev(a,b,c) => Teev(a,b,c),
            Tvve(a,b,c) => Tvve(a,b,c),
            Tvev(a,b,c) => Tvev(a,b,c),
            Tevv(a,b,c) => Tevv(a,b,c),
            Tvvv(a,b,c) => Tvvv(a,b,c),
            TNeee(a,b,c) => Teee(a,b,c),
            TNvee(a,b,c) => Tvee(a,b,c),
            TNeve(a,b,c) => Teve(a,b,c),
            TNeev(a,b,c) => Teev(a,b,c),
            TNvve(a,b,c) => Tvve(a,b,c),
            TNvev(a,b,c) => Tvev(a,b,c),
            TNevv(a,b,c) => Tevv(a,b,c),
            TNvvv(a,b,c) => Tvvv(a,b,c),
            Empty => Empty
        }
    }

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
            Empty => ("".to_string(), "".to_string(), "".to_string()),
            tri => tri.clone().invert().to_tuple()
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
            Empty => ("".to_string(), "".to_string(), "".to_string()),
            tri => tri.clone().invert().to_tuple_with_variable()
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
            TNeee(a,b,c) => format!("Not({},{},{})", a.to_string(),b.to_string(),c.to_string()),
            TNvee(a,b,c) => format!("Not({},{},{})", to_var(&a),b.to_string(),c.to_string()),
            TNeve(a,b,c) => format!("Not({},{},{})", a.to_string(),to_var(&b),c.to_string()),
            TNeev(a,b,c) => format!("Not({},{},{})", a.to_string(),b.to_string(), to_var(&c)),
            TNvve(a,b,c) => format!("Not({},{},{})", to_var(&a),to_var(&b),c.to_string()),
            TNvev(a,b,c) => format!("Not({},{},{})", to_var(&a),b.to_string(),to_var(&c)),
            TNevv(a,b,c) => format!("Not({},{},{})", a.to_string(),to_var(&b),to_var(&c)),
            TNvvv(a,b,c) => format!("Not({},{},{})", to_var(&a),to_var(&b),to_var(&c)),
            Empty => "Empty triplet".to_string()
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
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Teee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Tvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Teve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Teev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::Element(s3)))) => Ok((t, Language::Tri(Tvve(s1,s2,s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Tvev(s1,s2.into(),s3)))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Tevv(s1.into(),s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::Var(s3)))) => Ok((t, Language::Tri(Tvvv(s1,s2,s3)))),
        Err(e) => Err(e),
        // not first position
        Ok((t, (Language::NotElement(s1),Language::Element(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNeee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::NotVar(s1),Language::Element(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::NotElement(s1),Language::Var(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNeve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::NotElement(s1),Language::Element(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNeev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::NotVar(s1),Language::Var(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNvve(s1,s2,s3.into())))),
        Ok((t, (Language::NotVar(s1),Language::Element(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNvev(s1,s2.into(),s3)))),
        Ok((t, (Language::NotElement(s1),Language::Var(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNevv(s1.into(),s2,s3)))),
        Ok((t, (Language::NotVar(s1),Language::Var(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNvvv(s1,s2,s3)))),
        // not second position
        Ok((t, (Language::Element(s1),Language::NotElement(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNeee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::Var(s1),Language::NotElement(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::Element(s1),Language::NotVar(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNeve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::Element(s1),Language::NotElement(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNeev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::Var(s1),Language::NotVar(s2),Language::Element(s3)))) => Ok((t, Language::Tri(TNvve(s1,s2,s3.into())))),
        Ok((t, (Language::Var(s1),Language::NotElement(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNvev(s1,s2.into(),s3)))),
        Ok((t, (Language::Element(s1),Language::NotVar(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNevv(s1.into(),s2,s3)))),
        Ok((t, (Language::Var(s1),Language::NotVar(s2),Language::Var(s3)))) => Ok((t, Language::Tri(TNvvv(s1,s2,s3)))),
        // not third position
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::NotElement(s3)))) => Ok((t, Language::Tri(TNeee(s1.into(),s2.into(),s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::NotElement(s3)))) => Ok((t, Language::Tri(TNvee(s1,s2.into(),s3.into())))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::NotElement(s3)))) => Ok((t, Language::Tri(TNeve(s1.into(),s2,s3.into())))),
        Ok((t, (Language::Element(s1),Language::Element(s2),Language::NotVar(s3)))) => Ok((t, Language::Tri(TNeev(s1.into(),s2.into(),s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::NotElement(s3)))) => Ok((t, Language::Tri(TNvve(s1,s2,s3.into())))),
        Ok((t, (Language::Var(s1),Language::Element(s2),Language::NotVar(s3)))) => Ok((t, Language::Tri(TNvev(s1,s2.into(),s3)))),
        Ok((t, (Language::Element(s1),Language::Var(s2),Language::NotVar(s3)))) => Ok((t, Language::Tri(TNevv(s1.into(),s2,s3)))),
        Ok((t, (Language::Var(s1),Language::Var(s2),Language::NotVar(s3)))) => Ok((t, Language::Tri(TNvvv(s1,s2,s3)))),
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
