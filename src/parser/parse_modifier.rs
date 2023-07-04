pub use nom::{
    bytes::complete::tag,
    sequence::preceded,
    branch::alt,
    multi::many1,
    IResult,
};

pub use crate::parser::base_parser::{
    Language,
    Triplet,
    Triplet::*,
    parse_triplet_and,
};

use nom::Err;
use nom::error::Error;
use crate::parser::base_parser::PredicatAST;

type QueryAST = (Vec<Language>, Vec<Language>,Vec<Language>);
type QueryVarAST<'a> = (Vec<String>, Vec<&'a str>);

fn parse_delete_modifier(s: &str) -> IResult<&str, PredicatAST> {
    let res = preceded(tag("delete"),
        many1(parse_triplet_and)
    )(s);
    match res {
        Ok((s, v)) => Ok((s, PredicatAST::DeleteModifier(v))),
        Err(e) => Err(e)
    }
}
 
fn triplet_to_insert(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('{}','{}','{}')",
            tup.0, tup.1, tup.2)
}

fn parse_add_modifier(s: &str) -> IResult<&str, PredicatAST> {
    let res = preceded(tag("add"),
        many1(parse_triplet_and)
    )(s);
    match res {
        Ok((s, v)) => Ok((s, PredicatAST::AddModifier(v))),
        Err(e) => Err(e) 
    }
}

pub fn parse_modifier(s: &str) -> PredicatAST {
    let res: IResult<&str, PredicatAST> = alt((
            parse_add_modifier, 
            parse_delete_modifier  
        ))(s);
    match res {
        Ok((s, modifier)) => modifier,
        Err(e) => PredicatAST::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_add_modifier,
        parse_delete_modifier,
        triplet_to_insert,
        PredicatAST,
        Triplet::*,
        Language
    };

    #[test]
    fn test_add_modifier() {
        let (s, args) =  parse_add_modifier("add pierre ami jean").unwrap();
        assert_eq!(args, 
                PredicatAST::AddModifier(vec![Language::Tri(Twww("pierre".to_string(), "ami".to_string(), "jean".to_string()))])
            );
    } 

    #[test]
    fn test_triplet_to_insert() {
        assert_eq!(
            triplet_to_insert(&Twww("pierre".to_string(),"ami".to_string(),"jean".to_string())),
            "INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('pierre','ami','jean')".to_string());
    }


    #[test]
    fn test_delete_modifier() {
        assert_eq!(
            parse_delete_modifier("delete pierre ami jean").unwrap().1,
            PredicatAST::DeleteModifier(vec![Language::Tri(Twww("pierre".to_string(), "ami".to_string(), "jean".to_string()))]));

        //assert_eq!(
            //parse_delete_modifier("delete pierre ami jean and julie ami susanne").unwrap().1,
            //vec![Language::Tri(Twww("pierre".to_string(), "ami".to_string(), "jean".to_string())),
                 //Language::Tri(Twww("julie".to_string(), "ami".to_string(), "susanne".to_string()))]);
                 
    }

}
