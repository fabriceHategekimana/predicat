#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]
use parser;
use std::env;
use knowledge;
use knowledge::Cache;
use base_context::Context;
use simple_context::SimpleContext;
use crate::knowledge::Knowledgeable;
//use crate::knowledge::new_knowledge;
use metaprogramming::substitute_variables;
use knowledge::SqliteKnowledge;
use crate::parser::base_parser::PredicatAST;
use crate::parser::parse_command;



struct Interpreter {
    context: SimpleContext,
    knowledge: SqliteKnowledge
}

impl Interpreter {

    fn new(k: SqliteKnowledge) -> Interpreter {
        Interpreter {
            context: SimpleContext::new(),
            knowledge: k
        }
    }

    fn run(&mut self, cmd: &str) -> SimpleContext {
        //let cmd = self.get_args_or("add socrate est mortel");
        
        let mut context = self.interpret(&vec![cmd.to_string()], &self.knowledge);

        while context.has_commands() && !context.has_error() {
            context = self.interpret(&context.get_aftercmds(), &self.knowledge);
        }

        //context.display(); //display context or error
        self.context = context.clone();
        context
    }

    fn display(&self) -> () {
        self.context.display()
    }

    fn parse(command: &String) -> Vec<PredicatAST> {
        parse_command(command).iter()
                    .map(PredicatAST::clone)
                    .flat_map(substitute_variables(SimpleContext::new()))
                    .flatten().collect()
    }

    fn get_user_passed_arguments(&self) -> String {
        env::args().skip(1)
            .fold(String::new(), |acc, arg| format!("{}{} ", acc, &arg))
    }

    fn get_args_or(&self, query: &str) -> String {
        let args = self.get_user_passed_arguments();
        if args == "".to_string() {
            String::from(query)
        }
        else{
            args
        }
    }

    fn execute(&self, cmds: &[PredicatAST], knowledge: &impl Knowledgeable) -> Option<SimpleContext> {
        let context = knowledge
                .valid_commands(cmds.to_vec())?
                .iter()
                .map(|x| knowledge.execute_command(x))
                .reduce(SimpleContext::join_contexts)?;

        Some(context.clone())
    }

    fn interpret(&self, cmds: &[String], knowledge: &impl Knowledgeable) -> SimpleContext {
        let cmds = cmds.iter()
                    .flat_map(Self::parse)
                    .collect::<Vec<PredicatAST>>();
        self.execute(&cmds, knowledge)
                    .expect("Something went wrong")
    }

    fn clear(&self) -> () {
        self.knowledge.clear_all();
    }
}

impl Default for Interpreter {
   fn default() -> Interpreter {
       Interpreter {
           context: SimpleContext::new(),
           knowledge: SqliteKnowledge::new()
       }
   } 
}

fn main() {
        //let knowledge = new_knowledge("sqlite").expect("Can't open the knowledge!");
    //let interpreter = Interpreter::default();
    //interpreter.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add(){
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("add julien ami julie");
       assert_eq!(
           SimpleContext::from(vec![["julien", "ami", "julie"]]),
           interpreter.run("get julien ami julie"));
    }

    // TODO
    #[test]
    fn test_rule_ami() {
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("infer $A ami $B -> $B ami $A");
       interpreter.run("add julien ami julie");
        assert_eq!(
            SimpleContext::from(vec![["julien", "ami", "julie"],
                                    ["julie", "ami", "julien"]]),
            interpreter.run("get $A $B $C")
                  );
    }

}
