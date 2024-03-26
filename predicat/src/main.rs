#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]
use parser;
use std::env;
use knowledge;
use knowledge::Cache;
use base_context::Context;
use simple_context::SimpleContext;
use knowledge::Knowledgeable;
use metaprogramming::substitute_variables;
use knowledge::SqliteKnowledge;
use parser::base_parser::PredicatAST;
use parser::parse_command;
use knowledge::RuleManager;
use parser::base_parser::CommandType;
use parser::base_parser::Triplet;
use serial_test::serial;


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
        
        let mut context = self.interpret(&vec![cmd.to_string()], &self.knowledge);

        while context.has_commands() && !context.has_error() {
            context = self.interpret(&context.get_aftercmds(), &self.knowledge); } self.context = context.clone();
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
    
    fn get_rules(&self) -> Vec<String> {
        self.knowledge.get_rules()
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
    #[serial]
    fn test_add(){
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("add julien ami julie");
       assert_eq!(
           SimpleContext::from(vec![["julien", "ami", "julie"]]),
           interpreter.run("get julien ami julie"));
    }

    #[test]
    #[serial]
    fn test_rule_1() {
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("infer add $A ami $B -> add $B ami $A");
       interpreter.run("add julien ami julie");
       interpreter.get_rules();
       assert_eq!(interpreter.get_rules(),
           vec!["infer add $A ami $B"]);
    }

    #[test]
    #[serial]
    fn test_rule_2() {
       let mut interpreter = Interpreter::default();
       interpreter.clear();
       interpreter.run("infer add $A ami $B -> add $B ami $A");
       interpreter.run("add julien ami julie");
        assert_eq!(
            SimpleContext::from(vec![["julien", "ami", "julie"],
                                    ["julie", "ami", "julien"]]),
            interpreter.run("get $A $B $C")
                  );
    }

}
