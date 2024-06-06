#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]
// will be use for the language (AST) manipulation

use base_context::context_traits::{Context, Var};
use base_context::simple_context::SimpleContext;
use parser::base_parser::{PredicatAST, Triplet, Comp, Language};

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
        (None, Pos::First) => vec![Triplet::Tvee(v1.to_string(), ws.0.to_string(), ws.1.to_string())],
        (None, Pos::Second) => vec![Triplet::Teve(ws.0.to_string(), v1.to_string(), ws.1.to_string())],
        (None, Pos::Third) => vec![Triplet::Teev(ws.0.to_string(), ws.1.to_string(), v1.to_string())],
        (Some(v), Pos::First) => 
            v.iter().map(|x| Triplet::Teee(x.to_string(), ws.0.to_string(), ws.1.to_string())).collect(),
        (Some(v), Pos::Second) => 
            v.iter().map(|x| Triplet::Teee(ws.0.to_string(), x.to_string(), ws.1.to_string())).collect(),
        (Some(v), Pos::Third) => 
            v.iter().map(|x| Triplet::Teee(ws.0.to_string(), ws.1.to_string(), x.to_string())).collect()
    }
}

fn substitute_2_var_triplet(v1: &str, v2: &str, w: &str, doublepos: DoublePos, context: &SimpleContext) -> Vec<Triplet> {
    match (context.get_values(&v1), context.get_values(&v2), doublepos) {
        (None, None, DoublePos::FirstSecond) => vec![Triplet::Tvve(v1.to_string(), v2.to_string(), w.to_string())],
        (None, None, DoublePos::FirstThird) => vec![Triplet::Tvev(v1.to_string(), w.to_string(), v2.to_string())],
        (None, None, DoublePos::SecondThird) => vec![Triplet::Tevv(w.to_string(), v1.to_string(), v2.to_string())],
        (Some(v), None, DoublePos::FirstSecond) => 
           v.iter().map(|x| Triplet::Teve(x.to_string(), v2.to_string(), w.to_string())).collect(),
        (Some(v), None, DoublePos::FirstThird) => 
            v.iter().map(|x| Triplet::Teev(x.to_string(), w.to_string(), v2.to_string())).collect(),
        (Some(v), None, DoublePos::SecondThird) => 
            v.iter().map(|x| Triplet::Teev(w.to_string(), x.to_string(), v2.to_string())).collect(),
        (None, Some(v), DoublePos::FirstSecond) => 
            v.iter().map(|x| Triplet::Tvee(v1.to_string(), x.to_string(), w.to_string())).collect(),
        (None, Some(v), DoublePos::FirstThird) => 
            v.iter().map(|x| Triplet::Tvee(v1.to_string(), w.to_string(), x.to_string())).collect(),
        (None, Some(v), DoublePos::SecondThird) => 
            v.iter().map(|x| Triplet::Teve(w.to_string(), v1.to_string(), x.to_string())).collect(),
        (Some(v1), Some(v2), DoublePos::FirstSecond) => 
            v1.iter().zip(v2.iter()).map(|(x1,x2)| Triplet::Teee(x1.to_string(), x2.to_string(), w.to_string())).collect(),
        (Some(v1), Some(v2), DoublePos::FirstThird) => 
            v1.iter().zip(v2.iter()).map(|(x1,x2)| Triplet::Teee(x1.to_string(), w.to_string(), x2.to_string())).collect(),
        (Some(v1), Some(v2), DoublePos::SecondThird) => 
            v1.iter().zip(v2.iter()).map(|(x1,x2)| Triplet::Teee(w.to_string(), x1.to_string(), x2.to_string())).collect(),
    }
}

fn substitute_3_var_triplet(va1: &str, va2:&str, va3: &str, context: &SimpleContext) -> Vec<Triplet> {
   match (context.get_values(va1), context.get_values(va2), context.get_values(va3)) {
       (None, None, None) => vec![Triplet::Tvvv(va1.to_string(), va2.to_string(), va3.to_string())],
       (Some(v), None, None) => 
           v.iter().map(|x| Triplet::Tevv( x.to_string(), va2.to_string(), va3.to_string())).collect(),
       (None, Some(v), None) => 
           v.iter().map(|x| Triplet::Tvev(va1.to_string(), x.to_string(), va3.to_string())).collect(),
       (None, None, Some(v)) => 
           v.iter().map(|x| Triplet::Tvve(va1.to_string(), va2.to_string(), x.to_string())).collect(),
       (Some(v1), Some(v2), None) => 
           v1.iter().zip(v2.iter()).map(|(x1, x2)| Triplet::Teev(x1.to_string(), x2.to_string(), va3.to_string())).collect(),
       (Some(v1), None, Some(v2)) => 
           v1.iter().zip(v2.iter()).map(|(x1, x2)| Triplet::Teve(x1.to_string(), va2.to_string(), x2.to_string())).collect(),
       (None, Some(v1), Some(v2)) => 
           v1.iter().zip(v2.iter()).map(|(x1, x2)| Triplet::Teev(va1.to_string(), x2.to_string(), va2.to_string())).collect(),
       (Some(v1), Some(v2), Some(v3)) => 
           v1.iter().zip(v2.iter()).zip(v3.iter())
           .map(|((x1, x2), x3)| (x1, x2, x3))
           .map(|(x1, x2, x3)| Triplet::Teee(x1.to_string(), x2.to_string(), x3.to_string())).collect(),
   }
}

fn substitute_triplet(triplets: &[Triplet], context: &SimpleContext) -> Vec<Vec<Triplet>> {
    let vec = triplets.iter().map(|x| {
        match x {
            Triplet::Teee(w1, w2, w3) => vec![Triplet::Teee(w1.clone(), w2.clone(), w3.clone())],
            Triplet::Tvee(v1, w1, w2) => substitute_1_var_triplet(v1, (w1, w2), Pos::First, context),
            Triplet::Teve(w1, v1, w2) => substitute_1_var_triplet(v1, (w1, w2), Pos::Second, context),
            Triplet::Teev(w1, w2, v1) => substitute_1_var_triplet(v1, (w1, w2), Pos::Third, context),
            Triplet::Tvve(v1, v2, w1) => substitute_2_var_triplet(v1, v2, w1, DoublePos::FirstSecond, context),
            Triplet::Tvev(v1, w1, v2) => substitute_2_var_triplet(v1, v2, w1, DoublePos::FirstThird, context),
            Triplet::Tevv(w1, v1, v2) => substitute_2_var_triplet(v1, v2, w1, DoublePos::SecondThird, context),
            Triplet::Tvvv(v1, v2, v3) => substitute_3_var_triplet(v1, v2, v3, context),
            Triplet::Empty => vec![Triplet::Empty],
            Triplet::TNeee(w1, w2, w3) => vec![Triplet::TNeee(w1.clone(), w2.clone(), w3.clone())],
            Triplet::TNvee(v1, w1, w2) => substitute_1_var_triplet(v1, (w1, w2), Pos::First, context)
                                            .iter().map(|x| x.clone().invert()).collect(),
            Triplet::TNeve(w1, v1, w2) => substitute_1_var_triplet(v1, (w1, w2), Pos::Second, context)
                                            .iter().map(|x| x.clone().invert()).collect(),
            Triplet::TNeev(w1, w2, v1) => substitute_1_var_triplet(v1, (w1, w2), Pos::Third, context)
                                            .iter().map(|x| x.clone().invert()).collect(),
            Triplet::TNvve(v1, v2, w1) => substitute_2_var_triplet(v1, v2, w1, DoublePos::FirstSecond, context)
                                            .iter().map(|x| x.clone().invert()).collect(),
            Triplet::TNvev(v1, w1, v2) => substitute_2_var_triplet(v1, v2, w1, DoublePos::FirstThird, context)
                                            .iter().map(|x| x.clone().invert()).collect(),
            Triplet::TNevv(w1, v1, v2) => substitute_2_var_triplet(v1, v2, w1, DoublePos::SecondThird, context)
                                            .iter().map(|x| x.clone().invert()).collect(),
            Triplet::TNvvv(v1, v2, v3) => substitute_3_var_triplet(v1, v2, v3, context)
                                            .iter().map(|x| x.clone().invert()).collect(),
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
    false => (0..context.dataframe_len()).into_iter().map(|x| vec![]).collect()
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
        true => (0..context.dataframe_len()).into_iter().map(|x| v[0].clone()).collect(),
        false => v.clone()
    }
}

fn fullfill2<E: Clone>(v: Vec<E>, context: &SimpleContext) -> Vec<E> {
    match v.len() == 1 {
        true => (0..context.dataframe_len()).into_iter().map(|x| v[0].clone()).collect(),
        false => v.clone()
    }
}

fn substitute_query(vars: &[Var], triplets: &[Triplet], comps: &[Comp], context: &SimpleContext) -> Vec<PredicatAST> {
   match context.dataframe_len() {
       0 => vec![PredicatAST::Query((vars.to_vec(), triplets.to_vec(), comps.to_vec()))],
       _ => {
       let tripletss = substitute_triplet(triplets, context);
       let compss = substitute_comp(comps, context);
       let new_vars = vars.iter()
           .filter(|Var(var)| context.get_variables().iter().all(|x| x.0 != *var))
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

pub fn substitute_variables(context: SimpleContext) -> impl Fn(PredicatAST) -> Option<Vec<PredicatAST>> {
    move |ast: PredicatAST| -> Option<Vec<PredicatAST>> {
        if context.dataframe_len() == 0 {
            Some(vec![ast.clone()])
        } else {
            let res = match ast {
                PredicatAST::Query((vars, triplets, comps)) => substitute_query(&vars, &triplets, &comps, &context),
                PredicatAST::AddModifier(tri) => substitute_triplet_to_predicat_ast(&tri, PredicatAST::AddModifier, &context),
                PredicatAST::DeleteModifier(tri) => substitute_triplet_to_predicat_ast(&tri, PredicatAST::DeleteModifier, &context),
                x => vec![x.clone()]
            };
            Some(res)
        }
    }
}

