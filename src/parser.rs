#![allow(dead_code, unused_variables, unused_imports)]

mod parse_modifier;
mod parse_query;
mod base_parser;

use parse_query::{
    parse_query,
    alt
};
use parse_modifier::parse_modifier;

//main
pub fn parse_command(s: &str) -> Vec<String> {
    let res = alt((
            parse_query,
            parse_modifier))(s);
    match res {
        Ok((s, t)) => t,
        Err(e) => vec![String::from("Error")] 
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
