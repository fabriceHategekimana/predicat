#![allow(dead_code, unused_variables, unused_imports)]

mod parse_modifier;
mod parse_query;
mod base_parser;

use parse_modifier::parse_modifier;

use parse_query::{
    parse_query,
    alt
};

use self::base_parser::Language;

#[derive(PartialEq, Debug)]
pub enum PredicatAST<'a> {
    Query((Vec<Language<'a>>, Vec<Language<'a>>, Vec<Language<'a>>)),
    Modifier(Vec<String>),
    Empty
}

//main
pub fn parse_command(s: &str) -> PredicatAST {
    if &s[0..3] == "get" {
       match parse_query(s) {
           Ok((s, t)) => PredicatAST::Query(t),
           _ => PredicatAST::Empty
       }
    }
    else {
        match parse_modifier(s) {
            Ok((s, t)) => PredicatAST::Modifier(t),
            _ => PredicatAST::Empty
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        assert_eq!(parse_query("get $A such_as $A ami Bob $A == 7").unwrap().1,
                   //(vec![Var("A")], vec![Tri(Tvww("A","ami","Bob"))], vec![Comp(" $A == 7")])
                   vec!["SELECT A FROM (SELECT subject AS A FROM facts WHERE link='ami' AND goal='Bob') WHERE A = 7;"]
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7").unwrap().1,
                   //(vec![Var("A"), Var("B")], vec![Tri(Tvwv("A","ami","B"))], vec![Comp(" $A == 7")])
                   vec!["SELECT A,B FROM (SELECT subject AS A,goal AS B FROM facts WHERE link='ami') WHERE A = 7;"]
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7 and $B < 9").unwrap().1,
                   //(vec![Var("A"), Var("B")], vec![Tri(Tvwv("A","ami","B"))], vec![Comp(" $A == 7"), Comp(" $B < 9")])
                   vec!["SELECT A,B FROM (SELECT subject AS A,goal AS B FROM facts WHERE link='ami') WHERE A = 7 AND B < 9;"]
                   );
    }

}
