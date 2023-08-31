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

fn execute((kcmd, cmd): (String, PredicatAST), knowledge: &impl Knowledgeable) -> Option<SimpleContext> {
    match knowledge.is_invalid(&cmd){
        true => None,
        false => Some(knowledge.execute(&kcmd))
    }
}

fn after_execution(cmds: &[PredicatAST], knowledge: &impl Knowledgeable) {
        knowledge.get_commands_from(&cmds).iter()
        .map(|cmd| parse_and_execute(&cmd, knowledge, SimpleContext::new()))
        .fold(SimpleContext::new(), join_contexts);
}

fn parse_and_execute(command: &str, knowledge: &impl Knowledgeable, context: SimpleContext) -> SimpleContext {
    let translate_cmd = |x| (knowledge.translate(&x).unwrap_or("".to_string()), x);
    let execute_cmd = |x| execute(x, knowledge);

    parse_command(command).iter().fold(context, |ctx, ast| {
       let cmds = substitute(ast, &ctx);
       let final_context = cmds.clone().into_iter()
               .map(translate_cmd)
               .flat_map(execute_cmd)
               .fold(SimpleContext::new(), join_contexts);
       after_execution(&cmds, knowledge);
       final_context
    })
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
