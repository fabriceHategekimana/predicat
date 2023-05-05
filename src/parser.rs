#![allow(dead_code, unused_variables, unused_imports)]

mod parse_modifier;
mod parse_query;
pub mod base_parser;

use polars::frame::DataFrame;

use parse_query::{
    parse_query,
    alt
};

use parse_modifier::parse_modifier;

use self::base_parser::{Language, Triplet};

#[derive(PartialEq, Debug)]
pub enum PredicatAST<'a> {
    Query(
        (Vec<Language<'a>>,
         Vec<Language<'a>>,
         Vec<Language<'a>>)),
    Modifier(Vec<String>),
    Empty
}

//main
pub fn parse_command(string: &str, _context: DataFrame) -> Vec<PredicatAST> {
    // TODO use the _context variable to generate the needed informations
    string.split(" | ")
          .map(parse_query_and_modifier)
          .collect::<Vec<PredicatAST>>()
}

fn is_a_query(s: &str) -> bool {
    &s[0..3] == "get"
}

fn parse_query_and_modifier(s: &str) -> PredicatAST {
    if is_a_query(s) {
       parse_query(s)
    }
    else {
       parse_modifier(s)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        assert_eq!(parse_query("get $A such_as $A ami Bob $A == 7").unwrap().1,
                   (vec![Language::Var("A")], vec![Language::Tri(Triplet::Tvww("A","ami","Bob"))], vec![Language::Comp(" $A == 7")])
                   //vec!["SELECT A FROM (SELECT subject AS A FROM facts WHERE link='ami' AND goal='Bob') WHERE A = 7;"]
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7").unwrap().1,
                   (vec![Language::Var("A"), Language::Var("B")], vec![Language::Tri(Triplet::Tvwv("A","ami","B"))], vec![Language::Comp(" $A == 7")])
                   //vec!["SELECT A,B FROM (SELECT subject AS A,goal AS B FROM facts WHERE link='ami') WHERE A = 7;"]
                   );
        assert_eq!(parse_query("get $A $B such_as $A ami $B and $A == 7 and $B < 9").unwrap().1,
                   (vec![Language::Var("A"), Language::Var("B")], vec![Language::Tri(Triplet::Tvwv("A","ami","B"))], vec![Language::Comp(" $A == 7"), Language::Comp(" $B < 9")])
                   //vec!["SELECT A,B FROM (SELECT subject AS A,goal AS B FROM facts WHERE link='ami') WHERE A = 7 AND B < 9;"]
                   );
    }
}
