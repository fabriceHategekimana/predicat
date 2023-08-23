// will be use for the language (AST) manipulation

use base_context::Context;
use simple_context::SimpleContext;
use parser::base_parser::{PredicatAST, Var, Triplet, Comp, Language};

fn substitute_query(vars: &[Var], triplets: &[Triplet], comps: &[Comp]) -> Vec<PredicatAST> {
    todo!();
}

fn substitute_add_modifier(langs: &[Language]) -> Vec<PredicatAST> {
    todo!();
}

fn substitute_delete_modifier(langs: &[Language]) -> Vec<PredicatAST> {
    todo!();
}

pub fn substitute(ast: &PredicatAST, context: SimpleContext) -> Vec<PredicatAST> {
    match ast {
        PredicatAST::Query((vars, triplets, comps)) => substitute_query(vars, triplets, comps),
        PredicatAST::AddModifier(langs) => substitute_add_modifier(langs),
        PredicatAST::DeleteModifier(langs) => substitute_delete_modifier(langs),
        PredicatAST::Empty => vec![PredicatAST::Empty],
        PredicatAST::Debug(s) => vec![PredicatAST::Debug(s.clone())]
    };
    todo!();
}
