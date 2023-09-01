use parser;
use importer;
use knowledge;
use std::env;
use metaprogramming::substitute;

use crate::parser::parse_command;

use crate::knowledge::Knowledgeable;
use crate::knowledge::new_knowledge;
use base_context::Context;
use simple_context::SimpleContext;
use crate::parser::base_parser::PredicatAST;

fn get_user_passed_arguments() -> String {
    env::args().skip(1)
        .fold(String::new(), |acc, arg| format!("{}{} ", acc, &arg))
}

fn get_args_or(query: &str) -> String {
    let args = get_user_passed_arguments();
    if args == "".to_string() {
        String::from(query)
    }
    else{
        args
    }
}

fn join_contexts(ctx1: SimpleContext, ctx2: SimpleContext) -> SimpleContext {
    ctx1.join(ctx2)
}

fn filter_invalid((kcmd, cmd): (String, PredicatAST), knowledge: &impl Knowledgeable) -> Option<(String, PredicatAST)> {
    match knowledge.is_invalid(&cmd){
        true => { println!("{:?} is invalid", cmd); None },
        false => Some((kcmd, cmd))
    }
}

fn after_execution(cmds: &[PredicatAST], knowledge: &impl Knowledgeable) -> SimpleContext {
        cmds.iter()
        .flat_map(|x| knowledge.translate(x))
        .map(|cmd| parse_and_execute(&cmd, knowledge, SimpleContext::new()))
        .fold(SimpleContext::new(), join_contexts)
}


fn execute_command((octx, aftercmd): (Option<SimpleContext>, Vec<PredicatAST>), ast: &PredicatAST, kn: &impl Knowledgeable) -> (Option<SimpleContext>, Vec<PredicatAST>) {
    let abort = (None, vec![]);

    match octx {
        None => abort,
        Some(ctx) => {
            let cmds = substitute(ast, &ctx);
            match cmds.iter().any(|x| kn.is_invalid(x) == true) {
                true => abort ,
                false => (
                    Some(cmds.iter().flat_map(|cmd| kn.translate(cmd))
                        .map(|cmd| kn.execute(&cmd))
                        .fold(SimpleContext::new(), join_contexts)),
                    cmds.iter().flat_map(|cmd| kn.get_commands_from(cmd))
                    .flat_map(|cmd| parse_command(&cmd))
                    .chain(aftercmd.into_iter())
                    .collect())
            }
        }
    }
}

fn parse_and_execute(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> SimpleContext {

    let res = parse_command(command).iter()
        .fold((Some(context), vec![]), |entry, cmd| execute_command(entry, cmd, knowledge));

    if let (Some(_), cmds) = res {
        after_execution(&cmds, knowledge)
    } else {
        SimpleContext::new()
    }
}

// activation of a rule
// find an optimal query for a request
// only concrete element
// ex: add pierre ami beatrice
// possible matchs:
// subject = pierre or 
// link = ami or
// goal = beatrice

fn main() {
    //let command = get_args_or("add Socrate est mortel");
    let command = get_args_or("rule before add $A ami $B : add $B ami $A");
    let Ok(knowledge) = new_knowledge("sqlite") else {panic!("Can't open the knowledge!")};
    let context = SimpleContext::new();
    let res = parse_and_execute(&command, &knowledge, context);
    res.display();
}
