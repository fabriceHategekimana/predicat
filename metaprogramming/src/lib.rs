#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]
// will be use for the language (AST) manipulation

use base_context::Context;
use simple_context::SimpleContext;
use parser::base_parser::{PredicatAST, Var, Triplet, Comp, Language};

#[derive(PartialEq, Debug, Clone, Copy)]
enum Pos {
    First,
    Second,
    Third
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum DoublePos {
    FirstSecond,
    FirstThird,
    SecondThird,
}

fn substitute_1_var_triplet(v1: &str, ws: (&str, &str), pos: Pos, context: &SimpleContext) -> Vec<Triplet> {
    match (context.get_values(&v1), pos) {
        (None, Pos::First) => vec![Triplet::Tvww(v1.to_string(), ws.0.to_string(), ws.1.to_string())],
        (None, Pos::Second) => vec![Triplet::Twvw(ws.0.to_string(), v1.to_string(), ws.1.to_string())],
        (None, Pos::Third) => vec![Triplet::Twwv(ws.0.to_string(), ws.1.to_string(), v1.to_string())],
        (Some(v), Pos::First) => 
            v.iter().map(|x| Triplet::Twww(x.to_string(), ws.0.to_string(), ws.1.to_string())).collect(),
        (Some(v), Pos::Second) => 
            v.iter().map(|x| Triplet::Twww(ws.0.to_string(), x.to_string(), ws.1.to_string())).collect(),
        (Some(v), Pos::Third) => 
            v.iter().map(|x| Triplet::Twww(ws.0.to_string(), ws.1.to_string(), x.to_string())).collect()
    }
}

fn substitute_2_var_triplet(v1: &str, v2: &str, w: &str, doublepos: DoublePos, context: &SimpleContext) -> Vec<Triplet> {
    match (context.get_values(&v1), context.get_values(&v2), doublepos) {
        (None, None, DoublePos::FirstSecond) => vec![Triplet::Tvvw(v1.to_string(), v2.to_string(), w.to_string())],
        (None, None, DoublePos::FirstThird) => vec![Triplet::Tvwv(v1.to_string(), w.to_string(), v2.to_string())],
        (None, None, DoublePos::SecondThird) => vec![Triplet::Twvv(w.to_string(), v1.to_string(), v2.to_string())],
        (Some(v), None, DoublePos::FirstSecond) => 
           v.iter().map(|x| Triplet::Twvw(x.to_string(), v2.to_string(), w.to_string())).collect(),
        (Some(v), None, DoublePos::FirstThird) => 
            v.iter().map(|x| Triplet::Twwv(x.to_string(), w.to_string(), v2.to_string())).collect(),
        (Some(v), None, DoublePos::SecondThird) => 
            v.iter().map(|x| Triplet::Twwv(w.to_string(), x.to_string(), v2.to_string())).collect(),
        (None, Some(v), DoublePos::FirstSecond) => 
            v.iter().map(|x| Triplet::Tvww(v1.to_string(), x.to_string(), w.to_string())).collect(),
        (None, Some(v), DoublePos::FirstThird) => 
            v.iter().map(|x| Triplet::Tvww(v1.to_string(), w.to_string(), x.to_string())).collect(),
        (None, Some(v), DoublePos::SecondThird) => 
            v.iter().map(|x| Triplet::Twvw(w.to_string(), v1.to_string(), x.to_string())).collect(),
        (Some(v1), Some(v2), DoublePos::FirstSecond) => 
            v1.iter().zip(v2.iter()).map(|(x1,x2)| Triplet::Twww(x1.to_string(), x2.to_string(), w.to_string())).collect(),
        (Some(v1), Some(v2), DoublePos::FirstThird) => 
            v1.iter().zip(v2.iter()).map(|(x1,x2)| Triplet::Twww(x1.to_string(), w.to_string(), x2.to_string())).collect(),
        (Some(v1), Some(v2), DoublePos::SecondThird) => 
            v1.iter().zip(v2.iter()).map(|(x1,x2)| Triplet::Twww(w.to_string(), x1.to_string(), x2.to_string())).collect(),
    }
}

fn substitute_3_var_triplet(va1: &str, va2:&str, va3: &str, context: &SimpleContext) -> Vec<Triplet> {
   match (context.get_values(va1), context.get_values(va2), context.get_values(va3)) {
       (None, None, None) => vec![Triplet::Tvvv(va1.to_string(), va2.to_string(), va3.to_string())],
       (Some(v), None, None) => 
           v.iter().map(|x| Triplet::Twvv( x.to_string(), va2.to_string(), va3.to_string())).collect(),
       (None, Some(v), None) => 
           v.iter().map(|x| Triplet::Tvwv(va1.to_string(), x.to_string(), va3.to_string())).collect(),
       (None, None, Some(v)) => 
           v.iter().map(|x| Triplet::Tvvw(va1.to_string(), va2.to_string(), x.to_string())).collect(),
       (Some(v1), Some(v2), None) => 
           v1.iter().zip(v2.iter()).map(|(x1, x2)| Triplet::Twwv(x1.to_string(), x2.to_string(), va3.to_string())).collect(),
       (Some(v1), None, Some(v2)) => 
           v1.iter().zip(v2.iter()).map(|(x1, x2)| Triplet::Twvw(x1.to_string(), va2.to_string(), x2.to_string())).collect(),
       (None, Some(v1), Some(v2)) => 
           v1.iter().zip(v2.iter()).map(|(x1, x2)| Triplet::Twwv(va1.to_string(), x2.to_string(), va2.to_string())).collect(),
       (Some(v1), Some(v2), Some(v3)) => 
           v1.iter().zip(v2.iter()).zip(v3.iter())
           .map(|((x1, x2), x3)| (x1, x2, x3))
           .map(|(x1, x2, x3)| Triplet::Twww(x1.to_string(), x2.to_string(), x3.to_string())).collect(),
   }
}

fn substitute_triplet(triplets: &[Triplet], context: &SimpleContext) -> Vec<Triplet> {
    triplets.iter().flat_map(|x| {
        match x {
            Triplet::Twww(w1, w2, w3) => vec![Triplet::Twww(w1.clone(), w2.clone(), w3.clone())],
            Triplet::Tvww(v1, w1, w2) => substitute_1_var_triplet(v1, (w1, w2), Pos::First, context),
            Triplet::Twvw(w1, v1, w2) => substitute_1_var_triplet(v1, (w1, w2), Pos::Second, context),
            Triplet::Twwv(w1, w2, v1) => substitute_1_var_triplet(v1, (w1, w2), Pos::Third, context),
            Triplet::Tvvw(v1, v2, w1) => substitute_2_var_triplet(v1, v2, w1, DoublePos::FirstSecond, context),
            Triplet::Tvwv(v1, w1, v2) => substitute_2_var_triplet(v1, v2, w1, DoublePos::FirstThird, context),
            Triplet::Twvv(w1, v1, v2) => substitute_2_var_triplet(v1, v2, w1, DoublePos::SecondThird, context),
            Triplet::Tvvv(v1, v2, v3) => substitute_3_var_triplet(v1, v2, v3, context),
            Triplet::Empty => vec![Triplet::Empty]
        }
    }).collect()
}

fn format_comp(arg1: &str, arg2: &str, arg3: &str) -> Comp {
    Comp(format!("{} {} {}", arg1.to_string(), arg2.to_string(), arg3.to_string()))
}

fn substitute_comp(comps: &[Comp], context: SimpleContext) -> Vec<Comp> {
    comps.iter().flat_map(|comp| {
        let (val1, op, val2) = comp.get_content();
        match (context.get_values(&val1), context.get_values(&val2)) {
            (None, None) => 
                vec![format_comp(&val1, &op, &val2)],
            (Some(v), None) => 
                v.iter().map(|x| format_comp(x, &op, &val2)).collect(),
            (None, Some(v)) => 
                v.iter().map(|x| format_comp(&val1, &op, x)).collect(),
            (Some(v1), Some(v2)) => 
                v1.iter().zip(v2.iter()).map(|(x1, x2)| format_comp(x1, &op, x2)).collect()
        }
    }).collect()
}

fn substitute_query(vars: &[Var], triplets: &[Triplet], comps: &[Comp], context: SimpleContext) -> Vec<PredicatAST> {
    todo!();
   // The vars cant' be substituate (there are for extraction)
   // get A B where A age B and B < 18 and A ami C & context = (C = pierre, marc)
   // get A B where A age B and B < 18 and A ami pierre 
   // get A B where A age B and B < 18 and A ami marc 
   // TODO check the validity of the Language type (get variable should appear in the and elements)
}

fn substitute_add_modifier(langs: &[Language], context: SimpleContext) -> Vec<PredicatAST> {
    todo!();
}

fn substitute_delete_modifier(langs: &[Language], context: SimpleContext) -> Vec<PredicatAST> {
    todo!();
}

pub fn substitute(ast: &PredicatAST, context: SimpleContext) -> Vec<PredicatAST> {
    match ast {
        PredicatAST::Query((vars, triplets, comps)) => substitute_query(vars, triplets, comps, context),
        PredicatAST::AddModifier(langs) => substitute_add_modifier(langs, context),
        PredicatAST::DeleteModifier(langs) => substitute_delete_modifier(langs, context),
        PredicatAST::Empty => vec![PredicatAST::Empty],
        PredicatAST::Debug(s) => vec![PredicatAST::Debug(s.clone())]
    };
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_1_var_triplet1(){
       // C ami julie   |   context = (C = pierre, marc)
       // pierre ami julie 
       // marc ami julie 
        let mut context = SimpleContext::new();
        context = context.add_column("C", vec!["pierre".to_string(), "marc".to_string()]);
        assert_eq!(
            substitute_1_var_triplet("C", ("ami", "julie"), Pos::First, &context),
            vec![
            Triplet::Twww("pierre".to_string(), "ami".to_string(), "julie".to_string()), 
            Triplet::Twww("marc".to_string(), "ami".to_string(), "julie".to_string())
            ]);
    }

    #[test]
    fn test_substitute_1_var_triplet2(){
       // julie ami C   |   context = (C = pierre, marc)
       // julie ami pierre 
       // julie ami marc 
        let mut context = SimpleContext::new();
        context = context.add_column("C", vec!["pierre".to_string(), "marc".to_string()]);
        assert_eq!(
            substitute_1_var_triplet("C", ("julie", "ami"), Pos::Third, &context),
            vec![
            Triplet::Twww("julie".to_string(), "ami".to_string(), "pierre".to_string()), 
            Triplet::Twww("julie".to_string(), "ami".to_string(), "marc".to_string())
            ]);
    }

    #[test]
    fn test_substitute_2_var_triplet1(){
       // A ami C   |   context = (C = pierre, marc)
       // A ami pierre 
       // A ami marc 
        let mut context = SimpleContext::new();
        context = context.add_column("C", vec!["pierre".to_string(), "marc".to_string()]);
        assert_eq!(
            substitute_2_var_triplet("A", "C", "ami", DoublePos::FirstThird, &context),
            vec![
            Triplet::Tvww("A".to_string(), "ami".to_string(), "pierre".to_string()), 
            Triplet::Tvww("A".to_string(), "ami".to_string(), "marc".to_string())
            ]);
    }

    #[test]
    fn test_substitute_2_var_triplet2(){
       // A ami C   |   context = (C = pierre, marc, A = eva, sophie)
       // eva ami pierre 
       // sophie ami marc 
        let mut context = SimpleContext::new();
        context = context.add_column("C", vec!["pierre".to_string(), "marc".to_string()]);
        context = context.add_column("A", vec!["eva".to_string(), "sophie".to_string()]);
        assert_eq!(
            substitute_2_var_triplet("A", "C", "ami", DoublePos::FirstThird, &context),
            vec![
            Triplet::Twww("eva".to_string(), "ami".to_string(), "pierre".to_string()), 
            Triplet::Twww("sophie".to_string(), "ami".to_string(), "marc".to_string())
            ]);
    }

    #[test]
    fn test_substitute_3_var_triplet1(){
       // A B C   |   context = (C = pierre, marc, A = eva, sophie)
       // eva B pierre 
       // sophie B marc 
        let mut context = SimpleContext::new();
        context = context.add_column("C", vec!["pierre".to_string(), "marc".to_string()]);
        context = context.add_column("A", vec!["eva".to_string(), "sophie".to_string()]);
        assert_eq!(
            substitute_3_var_triplet("A", "B", "C", &context),
            vec![
            Triplet::Twvw("eva".to_string(), "B".to_string(), "pierre".to_string()), 
            Triplet::Twvw("sophie".to_string(), "B".to_string(), "marc".to_string())
            ]);
    }

    #[test]
    fn test_substitute_3_var_triplet2(){
       // A B C   |   context = (C = pierre, marc, A = eva, sophie, B = ami, collegue)
       // eva B pierre 
       // sophie B marc 
        let mut context = SimpleContext::new();
        context = context.add_column("C", vec!["pierre".to_string(), "marc".to_string()]);
        context = context.add_column("A", vec!["eva".to_string(), "sophie".to_string()]);
        context = context.add_column("B", vec!["ami".to_string(), "collegue".to_string()]);
        assert_eq!(
            substitute_3_var_triplet("A", "B", "C", &context),
            vec![
            Triplet::Twww("eva".to_string(), "ami".to_string(), "pierre".to_string()), 
            Triplet::Twww("sophie".to_string(), "collegue".to_string(), "marc".to_string())
            ]);
    }

    //#[test]
    //fn test_substitute_comp_1() {
        //assert_eq!(
            //substitute_comp(comps, context),
                  //);
    //}

}
