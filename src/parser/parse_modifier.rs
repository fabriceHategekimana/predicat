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

fn triplet_to_delete(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("DELETE FROM facts WHERE subject='{}' AND link='{}' AND goal='{}'",
            tup.0, tup.1, tup.2)
}

fn delete_to_insert(l: &Language) -> String {
    match l {
        Language::Tri(tri) => triplet_to_delete(&tri),
        _ => String::from("")
    }
}

fn parse_delete_modifier(s: &str) -> IResult<&str,Vec<String>> {
    let res = preceded(tag("delete"),
        many1(parse_triplet_and)
    )(s);
    match res {
        Ok((s, v)) => Ok((s, v.iter().map(|x| delete_to_insert(x)).collect::<Vec<String>>())),
        Err(e) => Err(e)
    }
}

fn triplet_to_insert(tri: &Triplet) -> String {
    let tup = tri.to_tuple_with_variable();
    format!("INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('{}','{}','{}')",
            tup.0, tup.1, tup.2)
}

fn add_to_insert(l: &Language) -> String {
    match l {
        Language::Tri(tri) => triplet_to_insert(&tri),
        _ => String::from("")
    }
}

fn parse_add_modifier(s: &str) -> IResult<&str, Vec<String>> {
    let res = preceded(tag("add"),
        many1(parse_triplet_and)
    )(s);
    match res {
        Ok((s, v)) => Ok((s, v.iter().map(|x| add_to_insert(x)).collect::<Vec<String>>())),
        Err(e) => Err(e)
    }
}

pub fn parse_modifier(s: &str) -> PredicatAST {
    let res: IResult<&str, Vec<String>> = alt((
            parse_add_modifier, 
            parse_delete_modifier  
        ))(s);
    match res {
        Ok((s,vs)) => PredicatAST::Modifier(vs),
        Err(e) => PredicatAST::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_add_modifier,
        parse_delete_modifier,
        triplet_to_insert,
        Triplet::*,
    };

    #[test]
    fn test_add_modifier() {
        assert_eq!(
            parse_add_modifier("add pierre ami jean").unwrap().1[0],
            "INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('pierre','ami','jean')"
                  );

        //assert_eq!(
            //parse_add_modifier("add pierre ami jean and julie ami susanne").unwrap().1,
            //vec!["INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('pierre','ami','jean')",
                 //"INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('julie','ami','susanne')"
            //]);
    } 

    #[test]
    fn test_triplet_to_insert() {
        assert_eq!(
            triplet_to_insert(&Twww("pierre".to_string(),"ami".to_string(),"jean".to_string())),
            "INSERT or IGNORE INTO facts (subject,link,goal) VALUES ('pierre','ami','jean')".to_string());
    }


    //#[test]
    //fn test_delete_modifier() {
        //assert_eq!(
            //parse_delete_modifier("delete pierre ami jean").unwrap().1,
            //vec!["DELETE FROM facts WHERE subject='pierre' AND link='ami' AND goal='jean'"]);
//
        //assert_eq!(
            //parse_delete_modifier("delete pierre ami jean and julie ami susanne").unwrap().1,
            //vec!["DELETE FROM facts WHERE subject='pierre' AND link='ami' AND goal='jean'",
                 //"DELETE FROM facts WHERE subject='julie' AND link='ami' AND goal='susanne'"]);
    //}

}
