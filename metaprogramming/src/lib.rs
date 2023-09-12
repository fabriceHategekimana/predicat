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

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>>
where
    T: Clone,
{
    assert!(!v.is_empty());
    (0..v[0].len())
        .map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>())
        .collect()
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

fn substitute_triplet(triplets: &[Triplet], context: &SimpleContext) -> Vec<Vec<Triplet>> {
    let vec = triplets.iter().map(|x| {
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
    }).map(|x| fullfill2(x, context)).collect();
    transpose(vec)
}

fn format_comp(arg1: &str, arg2: &str, arg3: &str) -> Comp {
    Comp(format!("{} {} {}", arg1.to_string(), arg2.to_string(), arg3.to_string()))
}

fn substitute_comp(comps: &[Comp], context: &SimpleContext) -> Vec<Vec<Comp>> {
    match comps != vec![] {
    true => {let vec = comps.iter().map(|comp| {
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
    }).collect();
    fullfill(transpose(vec), context)},
    false => (0..context.len()).into_iter().map(|x| vec![]).collect()
    }
}

fn substitute_query_helper(query: &PredicatAST, context: &SimpleContext) -> Vec<PredicatAST> {
    match query {
        PredicatAST::Query((a, b, c)) => substitute_query(&a, &b, &c, context),
        _ => vec![PredicatAST::Empty]
    }
}

fn fullfill<E: Clone>(v: Vec<Vec<E>>, context: &SimpleContext) -> Vec<Vec<E>> {
    match v.len() == 1 {
        true => (0..context.len()).into_iter().map(|x| v[0].clone()).collect(),
        false => v.clone()
    }
}

fn fullfill2<E: Clone>(v: Vec<E>, context: &SimpleContext) -> Vec<E> {
    match v.len() == 1 {
        true => (0..context.len()).into_iter().map(|x| v[0].clone()).collect(),
        false => v.clone()
    }
}

fn substitute_query(vars: &[Var], triplets: &[Triplet], comps: &[Comp], context: &SimpleContext) -> Vec<PredicatAST> {
   match context.len() {
       0 => vec![PredicatAST::Query((vars.to_vec(), triplets.to_vec(), comps.to_vec()))],
       _ => {
       let tripletss = substitute_triplet(triplets, context);
       let compss = substitute_comp(comps, context);
       let new_vars = vars.iter()
           .filter(|Var(var)| context.get_variables().iter().all(|x| x != var))
           .map(Var::clone)
           .collect::<Vec<_>>();
       tripletss.iter().zip(compss.iter())
           .map(|(triplets, comps)| PredicatAST::Query((new_vars.to_vec(), triplets.clone(), comps.clone()))).collect()
        }
   }
}

fn substitute_triplet_to_predicat_ast<C>(triplets: &[Triplet], constructor: C, context: &SimpleContext) -> Vec<PredicatAST> 
where C : Fn(Vec<Triplet>) -> PredicatAST {
    substitute_triplet(triplets, context).iter()
        .map(|x| constructor(x.clone())).collect()
}

pub fn substitute(ast: &PredicatAST, context: &SimpleContext) -> Option<Vec<PredicatAST>> {
    if context.len() == 0 {
        Some(vec![ast.clone()])
    } else {
        let res = match ast {
            PredicatAST::Query((vars, triplets, comps)) => substitute_query(vars, triplets, comps, context),
            PredicatAST::AddModifier(tri) => substitute_triplet_to_predicat_ast(tri, PredicatAST::AddModifier, context),
            PredicatAST::DeleteModifier(tri) => substitute_triplet_to_predicat_ast(tri, PredicatAST::DeleteModifier, context),
            x => vec![x.clone()]
        };
        Some(res)
    }
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
        context = context.add_column("C", &["pierre", "marc"]);
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
        context = context.add_column("C", &["pierre", "marc"]);
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
        context = context.add_column("C", &["pierre", "marc"]);
        assert_eq!(
            substitute_2_var_triplet("A", "C", "ami", DoublePos::FirstThird, &context),
            vec![
            Triplet::Tvww("A".to_string(), "ami".to_string(), "pierre".to_string()), 
            Triplet::Tvww("A".to_string(), "ami".to_string(), "marc".to_string())
            ]);
    }

    #[test]
    fn test_substitute_triplet() {
        // A ami C, (C: pierre, marc)
        let mut context = SimpleContext::new();
        context = context.add_column("C", &["pierre", "marc"]);
        assert_eq!(
                substitute_triplet(&[Triplet::Tvwv("A".to_string(), "ami".to_string(), "C".to_string())], &context),
                vec![
                vec![Triplet::Tvww("A".to_string(), "ami".to_string(), "pierre".to_string())], 
                vec![Triplet::Tvww("A".to_string(), "ami".to_string(), "marc".to_string())]
                ]);
    }

    #[test]
    fn test_substitute_triplet2() {
        // A ami C, (C: pierre, marc)
        let mut context = SimpleContext::new();
        context = context.add_column("C", &["pierre", "marc"]);
        assert_eq!(
                substitute_triplet(
                            &[Triplet::Tvwv( "A".to_string(), "age".to_string(), "B".to_string(),), Triplet::Tvwv( "A".to_string(), "ami".to_string(), "C".to_string(),)],
                            &context),
                vec![
                    vec![
                        Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                        Triplet::Tvww("A".to_string(), "ami".to_string(), "pierre".to_string())
                    ], 
                    vec![
                        Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                        Triplet::Tvww("A".to_string(), "ami".to_string(), "marc".to_string())
                    ], 
                ]);
    }

    #[test]
    fn test_substitute_2_var_triplet2(){
       // A ami C   |   context = (C = pierre, marc, A = eva, sophie)
       // eva ami pierre 
       // sophie ami marc 
        let mut context = SimpleContext::new();
        context = context.add_column("C", &["pierre", "marc"]);
        context = context.add_column("A", &["eva", "sophie"]);
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
        context = context.add_column("C", &["pierre", "marc"]);
        context = context.add_column("A", &["eva", "sophie"]);
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
        context = context.add_column("C", &["pierre", "marc"]);
        context = context.add_column("A", &["eva", "sophie"]);
        context = context.add_column("B", &["ami", "collegue"]);
        assert_eq!(
            substitute_3_var_triplet("A", "B", "C", &context),
            vec![
            Triplet::Twww("eva".to_string(), "ami".to_string(), "pierre".to_string()), 
            Triplet::Twww("sophie".to_string(), "collegue".to_string(), "marc".to_string())
            ]);
    }

    #[test]
    fn test_substitute_comp_1() {
       // A == C   |   context = (C = pierre, marc, A = eva, sophie, B = ami, collegue)
       // eva == pierre 
       // sophie == marc 
       let comps = [Comp("A == C".to_string())];
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
       context = context.add_column("A", &["eva", "sophie"]);
       context = context.add_column("B", &["ami", "collegue"]);
       assert_eq!(
           substitute_comp(&comps, &context),
           vec![vec![Comp("eva == pierre".to_string())], vec![Comp("sophie == marc".to_string())]]
                 );
    }

    #[test]
    fn test_substitute_query1() {
       // get A B where A age B and B < 18 and A ami C & context = (C = pierre, marc)
       // get A B where A age B and B < 18 and A ami pierre 
       // get A B where A age B and B < 18 and A ami marc 
       let query = PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                         Triplet::Tvwv("A".to_string(), "ami".to_string(), "C".to_string())],
                    vec![Comp("B < 18".to_string())]));
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
       assert_eq!(
            substitute_query_helper(&query, &context),
            vec![
            PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                         Triplet::Tvww("A".to_string(), "ami".to_string(), "pierre".to_string())],
                    vec![Comp("B < 18".to_string())])),
            PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                         Triplet::Tvww("A".to_string(), "ami".to_string(), "marc".to_string())],
                    vec![Comp("B < 18".to_string())]))
            ]
                  );
    }

    #[test]
    fn test_substitute_query2() {
       // get A where A ami C & context = (C = pierre, marc)
       // get A where A ami pierre 
       // get A where A age marc
       let query = PredicatAST::Query((
                    vec![Var("A".to_string())],
                    vec![Triplet::Tvwv("A".to_string(), "ami".to_string(), "C".to_string())],
                    vec![]));
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
       assert_eq!(
            substitute_query_helper(&query, &context),
            vec![
                   PredicatAST::Query((
                    vec![Var("A".to_string())],
                    vec![Triplet::Tvww("A".to_string(), "ami".to_string(), "pierre".to_string())],
                    vec![])),
                   PredicatAST::Query((
                    vec![Var("A".to_string())],
                    vec![Triplet::Tvww("A".to_string(), "ami".to_string(), "marc".to_string())],
                    vec![]))
            ]
                  );
    }

    #[test]
    fn test_substitute_query3() {
       // get A B C where A B C & context = (C = pierre, marc)
       // get A B where A B pierre 
       // get A B where A B marc
       let query = PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                    vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                    vec![]));
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
       assert_eq!(
            substitute_query_helper(&query, &context),
            vec![
                   PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvvw("A".to_string(), "B".to_string(), "pierre".to_string())],
                    vec![])),
                   PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvvw("A".to_string(), "B".to_string(), "marc".to_string())],
                    vec![]))
            ]
                  );
    }

    #[test]
    fn test_substitute_query_with_void_context() {
       // get A B C where A B C & context ()
       // get A B where A B C 
       let query = PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                    vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                    vec![]));
       let context = SimpleContext::new();
       assert_eq!(
            substitute_query_helper(&query, &context),
            vec![
                   PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string()), Var("C".to_string())],
                    vec![Triplet::Tvvv("A".to_string(), "B".to_string(), "C".to_string())],
                    vec![])),
            ]
                  );
    }

    #[test]
    fn test_fullfill() {
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
        assert_eq!(
            fullfill(vec![vec![7]], &context),
            vec![vec![7], vec![7]]
                  );
    }

    // add A ami B and B ami C ; (C = pierre, marc)
    // add A ami B and B ami pierre
    // add A ami B and B ami marc
    
    #[test]
    fn test_substitute_add_modifier() {
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
       let add_mod = vec![
                                Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string()),
                                Triplet::Tvwv("B".to_string(), "ami".to_string(), "C".to_string())];
        assert_eq!(
                substitute_triplet_to_predicat_ast(&add_mod, PredicatAST::AddModifier, &context),
                vec![
                PredicatAST::AddModifier(vec![
                                Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string()),
                                Triplet::Tvww("B".to_string(), "ami".to_string(), "pierre".to_string())]),
                PredicatAST::AddModifier(vec![
                                Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string()),
                                Triplet::Tvww("B".to_string(), "ami".to_string(), "marc".to_string())]),

                ]
                  );
    }

    fn test_substitute1() {
       // get A B where A age B and B < 18 and A ami C & context = (C = pierre, marc)
       // get A B where A age B and B < 18 and A ami pierre 
       // get A B where A age B and B < 18 and A ami marc 
       let query = PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                         Triplet::Tvwv("A".to_string(), "ami".to_string(), "C".to_string())],
                    vec![Comp("B < 18".to_string())]));
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
       assert_eq!(
            substitute(&query, &context),
            Some(vec![
            PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                         Triplet::Tvww("A".to_string(), "ami".to_string(), "pierre".to_string())],
                    vec![Comp("B < 18".to_string())])),
            PredicatAST::Query((
                    vec![Var("A".to_string()), Var("B".to_string())],
                    vec![Triplet::Tvwv("A".to_string(), "age".to_string(), "B".to_string()),
                         Triplet::Tvww("A".to_string(), "ami".to_string(), "marc".to_string())],
                    vec![Comp("B < 18".to_string())]))
            ])
                  );
    }

    #[test]
    fn test_substitute2() {
       let mut context = SimpleContext::new();
       context = context.add_column("C", &["pierre", "marc"]);
       let add_mod = PredicatAST::AddModifier(vec![
                                Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string()),
                                Triplet::Tvwv("B".to_string(), "ami".to_string(), "C".to_string())]);
        assert_eq!(
                substitute(&add_mod, &context),
                Some(vec![
                PredicatAST::AddModifier(vec![
                                Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string()),
                                Triplet::Tvww("B".to_string(), "ami".to_string(), "pierre".to_string())]),
                PredicatAST::AddModifier(vec![
                                Triplet::Tvwv("A".to_string(), "ami".to_string(), "B".to_string()),
                                Triplet::Tvww("B".to_string(), "ami".to_string(), "marc".to_string())]),

                ])
                  );
    }

}
